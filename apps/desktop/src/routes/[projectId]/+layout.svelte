<script lang="ts">
	import { goto } from '$app/navigation';
	import AnalyticsMonitor from '$components/AnalyticsMonitor.svelte';
	import Chrome from '$components/Chrome.svelte';
	import FileMenuAction from '$components/FileMenuAction.svelte';
	import FullviewLoading from '$components/FullviewLoading.svelte';
	import IrcPopups from '$components/IrcPopups.svelte';
	import NoBaseBranch from '$components/NoBaseBranch.svelte';
	import NotOnGitButlerBranch from '$components/NotOnGitButlerBranch.svelte';
	import ProblemLoadingRepo from '$components/ProblemLoadingRepo.svelte';
	import ProjectSettingsMenuAction from '$components/ProjectSettingsMenuAction.svelte';
	import ReduxResult from '$components/ReduxResult.svelte';
	import { BASE_BRANCH } from '$lib/baseBranch/baseBranch';
	import { BASE_BRANCH_SERVICE } from '$lib/baseBranch/baseBranchService.svelte';
	import { BRANCH_SERVICE } from '$lib/branches/branchService.svelte';
	import { SETTINGS_SERVICE } from '$lib/config/appSettingsV2';
	import {
		StackingReorderDropzoneManagerFactory,
		STACKING_REORDER_DROPZONE_MANAGER_FACTORY
	} from '$lib/dragging/stackingReorderDropzoneManager';
	import { FEED_FACTORY } from '$lib/feed/feed';
	import { FocusManager, FOCUS_MANAGER } from '$lib/focus/focusManager.svelte';
	import { DEFAULT_FORGE_FACTORY } from '$lib/forge/forgeFactory.svelte';
	import { GITHUB_CLIENT } from '$lib/forge/github/githubClient';
	import { GITLAB_CLIENT } from '$lib/forge/gitlab/gitlabClient.svelte';
	import { GitLabState, GITLAB_STATE } from '$lib/forge/gitlab/gitlabState.svelte';
	import { GIT_SERVICE } from '$lib/git/gitService';
	import { MODE_SERVICE } from '$lib/mode/modeService';
	import { showError, showInfo, showWarning } from '$lib/notifications/toasts';
	import { PROJECTS_SERVICE } from '$lib/project/projectsService';
	import { SECRET_SERVICE } from '$lib/secrets/secretsService';
	import { ID_SELECTION } from '$lib/selection/idSelection.svelte';
	import { UNCOMMITTED_SERVICE } from '$lib/selection/uncommittedService.svelte';
	import { STACK_SERVICE } from '$lib/stacks/stackService.svelte';
	import { CLIENT_STATE } from '$lib/state/clientState.svelte';
	import { combineResults } from '$lib/state/helpers';
	import { debounce } from '$lib/utils/debounce';
	import { WORKTREE_SERVICE } from '$lib/worktree/worktreeService.svelte';
	import { inject, provide } from '@gitbutler/shared/context';
	import { onDestroy, untrack, type Snippet } from 'svelte';
	import type { LayoutData } from './$types';

	const { data, children: pageChildren }: { data: LayoutData; children: Snippet } = $props();

	const { projectId, userService, posthog } = $derived(data);

	const baseBranchService = inject(BASE_BRANCH_SERVICE);
	const repoInfoResponse = $derived(baseBranchService.repo(projectId));
	const repoInfo = $derived(repoInfoResponse.current.data);
	const baseBranchResponse = $derived(baseBranchService.baseBranch(projectId));
	const baseBranch = $derived(baseBranchResponse.current.data);
	const pushRepoResponse = $derived(baseBranchService.pushRepo(projectId));
	const forkInfo = $derived(pushRepoResponse.current.data);
	const baseBranchName = $derived(baseBranch?.shortName);
	const branchService = inject(BRANCH_SERVICE);

	const stackService = inject(STACK_SERVICE);
	const feedFactory = inject(FEED_FACTORY);

	const secretService = inject(SECRET_SERVICE);
	const gitLabState = $derived(new GitLabState(secretService, repoInfo, projectId));
	$effect.pre(() => {
		provide(GITLAB_STATE, gitLabState);
	});
	const gitLabClient = inject(GITLAB_CLIENT);
	$effect.pre(() => {
		gitLabClient.set(gitLabState);
	});

	const user = $derived(userService.user);
	const accessToken = $derived($user?.github_access_token);

	const gitHubClient = inject(GITHUB_CLIENT);
	$effect.pre(() => gitHubClient.setToken(accessToken));
	$effect.pre(() => gitHubClient.setRepo({ owner: repoInfo?.owner, repo: repoInfo?.name }));

	const projectsService = inject(PROJECTS_SERVICE);
	const projectsResult = $derived(projectsService.projects());
	const projects = $derived(projectsResult.current.data);

	$effect.pre(() => {
		const stackingReorderDropzoneManagerFactory = new StackingReorderDropzoneManagerFactory(
			projectId,
			stackService
		);

		provide(STACKING_REORDER_DROPZONE_MANAGER_FACTORY, stackingReorderDropzoneManagerFactory);
	});

	$effect.pre(() => {
		provide(BASE_BRANCH, baseBranch);
	});

	const focusManager = new FocusManager();
	provide(FOCUS_MANAGER, focusManager);

	let intervalId: any;

	const forgeFactory = inject(DEFAULT_FORGE_FACTORY);

	// Refresh base branch if git fetch event is detected.
	const modeService = inject(MODE_SERVICE);
	const mode = $derived(modeService.mode({ projectId }));
	const head = $derived(modeService.head({ projectId }));

	const debouncedBaseBranchRefresh = debounce(async () => {
		await baseBranchService.refreshBaseBranch(projectId);
	}, 500);

	const debouncedRemoteBranchRefresh = debounce(async () => {
		await branchService.refresh().catch((error) => {
			console.error('Failed to refresh remote branches:', error);
		});
	}, 500);

	// TODO: Refactor `$head` into `.onHead()` as well.
	const gitService = inject(GIT_SERVICE);
	$effect(() =>
		gitService.onFetch(data.projectId, () => {
			debouncedBaseBranchRefresh();
			debouncedRemoteBranchRefresh();
		})
	);

	$effect(() => {
		if (baseBranch || head.current) debouncedRemoteBranchRefresh();
	});

	const gitlabConfigured = $derived(gitLabState.configured);

	$effect(() => {
		forgeFactory.setConfig({
			repo: repoInfo,
			pushRepo: forkInfo,
			baseBranch: baseBranchName,
			githubAuthenticated: !!$user?.github_access_token,
			gitlabAuthenticated: !!$gitlabConfigured,
			forgeOverride: projects?.find((project) => project.id === projectId)?.forge_override
		});
	});

	$effect(() => {
		posthog.setPostHogRepo(repoInfo);
		return () => {
			posthog.setPostHogRepo(undefined);
		};
	});

	// Once on load and every time the project id changes
	$effect(() => {
		if (projectId) {
			untrack(() => setupFetchInterval());
		} else {
			goto('/onboarding');
		}
	});

	async function fetchRemoteForProject() {
		await baseBranchService.fetchFromRemotes(projectId, 'auto');
	}

	function setupFetchInterval() {
		const autoFetchIntervalMinutes = $settingsStore?.fetch.autoFetchIntervalMinutes || 15;
		if (autoFetchIntervalMinutes < 0) {
			return;
		}
		fetchRemoteForProject();
		clearFetchInterval();
		const intervalMs = autoFetchIntervalMinutes * 60 * 1000; // 15 minutes
		intervalId = setInterval(async () => {
			await fetchRemoteForProject();
		}, intervalMs);

		return () => clearFetchInterval();
	}

	function clearFetchInterval() {
		if (intervalId) clearInterval(intervalId);
	}

	const settingsService = inject(SETTINGS_SERVICE);
	const settingsStore = settingsService.appSettings;

	onDestroy(() => {
		clearFetchInterval();
	});

	async function setActiveProjectOrRedirect() {
		const dontShowAgainKey = `git-filters--dont-show-again--${projectId}`;
		// Optimistically assume the project is viewable
		try {
			const info = await projectsService.setActiveProject(projectId);

			if (!info) {
				// The project is not found do nothing. The application will redirect to the start.
				return;
			}

			if (!info.is_exclusive) {
				showInfo(
					'Just FYI, this project is already open in another window',
					'There might be some unexpected behavior if you open it in multiple windows'
				);
			}

			if (info.db_error) {
				showError('The database was corrupted', info.db_error);
			}

			if (info.headsup && localStorage.getItem(dontShowAgainKey) !== '1') {
				showWarning('Important PSA', info.headsup, {
					label: "Don't show again",
					onClick: (dismiss) => {
						localStorage.setItem(dontShowAgainKey, '1');
						dismiss();
					}
				});
			}
		} catch (error: unknown) {
			showError('Failed to set the project active', error);
		}
	}

	$effect(() => {
		setActiveProjectOrRedirect();
	});

	// Clear the backend API when the project id changes.
	const clientState = inject(CLIENT_STATE);
	$effect(() => {
		if (projectId) {
			clientState.backendApi.util.resetApiState();
		}
	});

	// TODO: Can we combine WorktreeService and UncommittedService? The former
	// is powered by RTKQ, while the latter is a custom slice, but perhaps
	// they could be still be contained within the same .svelte.ts file.
	const worktreeService = inject(WORKTREE_SERVICE);
	const uncommittedService = inject(UNCOMMITTED_SERVICE);
	const worktreeDataResult = $derived(worktreeService.worktreeData(projectId));
	const worktreeData = $derived(worktreeDataResult.current.data);

	// This effect is a sort of bridge between rtkq and the custom slice.
	$effect(() => {
		if (worktreeData) {
			untrack(() => {
				uncommittedService.updateData({
					changes: worktreeData.rawChanges,
					// If assignments are not enabled we override the stack id to prevent
					// files from becoming hidden when toggling on/off.
					assignments: worktreeData.hunkAssignments
				});
			});
		}
	});

	// Here we clear any expired file selections. Note that the notion of
	// file selection is related to selecting things with checkmarks, and
	// that this `IdSelection` class should be deprecated in favor of
	// combining it with `UncommittedService`.
	const idSelection = inject(ID_SELECTION);
	const affectedPaths = $derived(worktreeData?.rawChanges.map((c) => c.path));
	$effect(() => {
		if (affectedPaths) {
			untrack(() => {
				idSelection.retain(affectedPaths);
			});
		}
	});

	// Listen for stack details updates from the backend.
	$effect(() => {
		stackService.stackDetailsUpdateListener(projectId);
	});

	// Listen for the feed updates from the backend.
	$effect(() => {
		feedFactory.getFeed(projectId);
	});
</script>

<ProjectSettingsMenuAction {projectId} />
<FileMenuAction />

<ReduxResult {projectId} result={combineResults(baseBranchResponse.current, mode.current)}>
	{#snippet children([baseBranch, mode], { projectId })}
		{#if !baseBranch}
			<NoBaseBranch {projectId} />
		{:else if baseBranch}
			{#if mode.type === 'OpenWorkspace' || mode.type === 'Edit'}
				<div class="view-wrap" role="group" ondragover={(e) => e.preventDefault()}>
					<Chrome {projectId} sidebarDisabled={mode.type === 'Edit'}>
						{@render pageChildren()}
					</Chrome>
				</div>
			{:else if mode.type === 'OutsideWorkspace'}
				<NotOnGitButlerBranch {projectId} {baseBranch}>
					{@render pageChildren()}
				</NotOnGitButlerBranch>
			{/if}
		{/if}
	{/snippet}
	{#snippet loading()}
		<FullviewLoading />
	{/snippet}
	{#snippet error(baseError)}
		<ProblemLoadingRepo {projectId} error={baseError} />
	{/snippet}
</ReduxResult>

<!-- {#if $settingsStore?.featureFlags.v3} -->
<IrcPopups />
<!-- {/if} -->

<AnalyticsMonitor {projectId} />

<style>
	.view-wrap {
		display: flex;
		position: relative;
		width: 100%;
	}
</style>
