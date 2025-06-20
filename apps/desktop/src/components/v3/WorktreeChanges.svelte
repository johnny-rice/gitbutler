<script lang="ts">
	import CardOverlay from '$components/CardOverlay.svelte';
	import ScrollableContainer from '$components/ConfigurableScrollableContainer.svelte';
	import Dropzone from '$components/Dropzone.svelte';
	import FileList from '$components/v3/FileList.svelte';
	import FileListMode from '$components/v3/FileListMode.svelte';
	import WorktreeChangesSelectAll from '$components/v3/WorktreeChangesSelectAll.svelte';
	import { createCommitStore } from '$lib/commits/contexts';
	import { UncommitDzHandler } from '$lib/commits/dropHandler';
	import { DefinedFocusable } from '$lib/focus/focusManager.svelte';
	import { focusable } from '$lib/focus/focusable.svelte';
	import { DiffService } from '$lib/hunks/diffService.svelte';
	import { AssignmentDropHandler } from '$lib/hunks/dropHandler';
	import { UncommittedService } from '$lib/selection/uncommittedService.svelte';
	import { StackService } from '$lib/stacks/stackService.svelte';
	import { UiState } from '$lib/state/uiState.svelte';
	import { TestId } from '$lib/testing/testIds';
	import { inject } from '@gitbutler/shared/context';
	import Badge from '@gitbutler/ui/Badge.svelte';
	import Button from '@gitbutler/ui/Button.svelte';
	import { isDefined } from '@gitbutler/ui/utils/typeguards';
	import { type Snippet } from 'svelte';
	import type { DropzoneHandler } from '$lib/dragging/handler';

	type Props = {
		projectId: string;
		stackId?: string;
		active: boolean;
		title: string;
		mode?: 'unassigned' | 'assigned';
		onDropzoneActivated?: (activated: boolean) => void;
		emptyPlaceholder?: Snippet;
	};

	let {
		projectId,
		active,
		stackId,
		title,
		mode = 'unassigned',
		onDropzoneActivated,
		emptyPlaceholder
	}: Props = $props();

	const [uiState, stackService, diffService, uncommittedService] = inject(
		UiState,
		StackService,
		DiffService,
		UncommittedService
	);

	const uncommitDzHandler = $derived(new UncommitDzHandler(projectId, stackService, uiState));

	const projectState = $derived(uiState.project(projectId));
	const drawerPage = $derived(projectState.drawerPage.get());
	const commitSourceId = $derived(projectState.commitSourceId.current);
	const isCommitting = $derived(drawerPage.current === 'new-commit' && commitSourceId === stackId);
	const stackState = $derived(stackId ? uiState.stack(stackId) : undefined);

	const defaultBranchResult = $derived(
		stackId !== undefined ? stackService.defaultBranch(projectId, stackId) : undefined
	);
	const defaultBranchName = $derived(defaultBranchResult?.current.data);

	const stacksResult = $derived(stackService.stacks(projectId));
	const stacks = $derived(stacksResult.current?.data || []);

	const changes = $derived(uncommittedService.changesByStackId(stackId || null));

	// TODO: Remove this after V3 transition complete.
	createCommitStore(undefined);

	let listMode: 'list' | 'tree' = $state('list');

	function startCommit() {
		if (changes.current) {
			uncommittedService.checkAll(stackId || null);
		}
		projectState.drawerPage.set('new-commit');
		projectState.commitSourceId.set(stackId);
		if (defaultBranchName) {
			projectState.stackId.set(stackId);
			stackState?.selection.set({ branchName: defaultBranchName });
		}
	}

	let scrollTopIsVisible = $state(true);
	let scrollBottomIsVisible = $state(true);

	const assignmentDZHandler = $derived(
		new AssignmentDropHandler(projectId, diffService, uncommittedService, stackId || null)
	);

	function getDropzoneLabel(handler: DropzoneHandler | undefined): string {
		if (handler instanceof UncommitDzHandler) {
			return 'Uncommit changes';
		} else if (mode === 'assigned') {
			return 'Assign changes';
		} else {
			return 'Unassign changes';
		}
	}
</script>

<Dropzone
	handlers={[uncommitDzHandler, assignmentDZHandler].filter(isDefined)}
	maxHeight
	onActivated={onDropzoneActivated}
>
	{#snippet overlay({ hovered, activated, handler })}
		<CardOverlay {hovered} {activated} label={getDropzoneLabel(handler)} />
	{/snippet}

	<div
		class="uncommitted-changes-wrap"
		use:focusable={{
			id: stackId
				? DefinedFocusable.UncommittedChanges + ':' + stackId
				: DefinedFocusable.UncommittedChanges,
			parentId: DefinedFocusable.ViewportLeft
		}}
	>
		{#if mode !== 'assigned' || changes.current.length > 0}
			<div
				data-testid={TestId.UncommittedChanges_Header}
				class="worktree-header"
				class:sticked-top={!scrollTopIsVisible}
			>
				<div class="worktree-header__general">
					{#if isCommitting}
						<WorktreeChangesSelectAll {stackId} />
					{/if}
					<div class="worktree-header__title truncate">
						<h3 class="text-14 text-semibold truncate">{title}</h3>
						<Badge>{changes.current.length}</Badge>
					</div>
				</div>
				{#if changes.current.length > 0}
					<FileListMode bind:mode={listMode} persist="uncommitted" />
				{/if}
			</div>
		{/if}

		{#if changes.current.length > 0}
			<ScrollableContainer
				autoScroll={false}
				onscrollTop={(visible) => {
					scrollTopIsVisible = visible;
				}}
				onscrollEnd={(visible) => {
					scrollBottomIsVisible = visible;
				}}
			>
				<div data-testid={TestId.UncommittedChanges_FileList} class="uncommitted-changes">
					<FileList
						draggableFiles
						selectionId={{ type: 'worktree', stackId }}
						showCheckboxes={isCommitting}
						changes={changes.current}
						{projectId}
						{listMode}
						{active}
						{stackId}
					/>
				</div>
			</ScrollableContainer>
			<div class="start-commit" class:sticked-bottom={!scrollBottomIsVisible}>
				<Button
					testId={TestId.StartCommitButton}
					kind={isCommitting ? 'outline' : 'solid'}
					type="button"
					wide
					disabled={defaultBranchResult?.current.isLoading}
					onclick={() => {
						if (isCommitting) {
							projectState.drawerPage.set(undefined);
						} else {
							startCommit();
						}
					}}
				>
					{#if isCommitting}
						Cancel committing
					{:else if mode === 'assigned' || stacks.length === 0}
						Start a commit…
					{:else}
						Commit to selected branch…
					{/if}
				</Button>
			</div>
		{:else}
			{@render emptyPlaceholder?.()}
		{/if}
	</div>
</Dropzone>

<style>
	.uncommitted-changes-wrap {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.worktree-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		height: 42px;
		padding: 10px 10px 10px 14px;
		gap: 8px;
		background-color: var(--clr-bg-1);
		text-wrap: nowrap;
		white-space: nowrap;
	}

	.worktree-header__general {
		display: flex;
		align-items: center;
		overflow: hidden;
		gap: 10px;
	}

	.worktree-header__title {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.uncommitted-changes {
		display: flex;
		flex: 1;
		flex-direction: column;
		width: 100%;
	}

	.start-commit {
		position: sticky;
		bottom: -1px;
		padding: 14px;
		background-color: var(--clr-bg-1);
	}

	/* MODIFIERS */
	.sticked-top {
		border-bottom: 1px solid var(--clr-border-2);
	}

	.sticked-bottom {
		border-top: 1px solid var(--clr-border-2);
	}
</style>
