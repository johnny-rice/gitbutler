<script lang="ts">
	import '$lib/styles/global.css';
	import { page } from '$app/state';
	import { ButlerAIClient, BUTLER_AI_CLIENT } from '$lib/ai/service';
	import { AUTH_SERVICE } from '$lib/auth/authService.svelte';
	import Footer from '$lib/components/Footer.svelte';
	import Navigation from '$lib/components/Navigation.svelte';
	import { OwnerService, OWNER_SERVICE } from '$lib/owner/ownerService';
	import { WebState, WEB_STATE } from '$lib/redux/store.svelte';
	import { SshKeyService, SSH_KEY_SERVICE } from '$lib/sshKeyService';
	import { UserService, USER_SERVICE } from '$lib/user/userService';
	import { BranchService, BRANCH_SERVICE } from '@gitbutler/shared/branches/branchService';
	import {
		LatestBranchLookupService,
		LATEST_BRANCH_LOOKUP_SERVICE
	} from '@gitbutler/shared/branches/latestBranchLookupService';
	import {
		ChatChannelsService,
		CHAT_CHANNELS_SERVICE
	} from '@gitbutler/shared/chat/chatChannelsService';
	import { inject, provide } from '@gitbutler/shared/context';
	import { FeedService, FEED_SERVICE } from '@gitbutler/shared/feeds/service';
	import { HttpClient, HTTP_CLIENT } from '@gitbutler/shared/network/httpClient';
	import {
		OrganizationService,
		ORGANIZATION_SERVICE
	} from '@gitbutler/shared/organizations/organizationService';
	import { ProjectService, PROJECT_SERVICE } from '@gitbutler/shared/organizations/projectService';
	import {
		RepositoryIdLookupService,
		REPOSITORY_ID_LOOKUP_SERVICE
	} from '@gitbutler/shared/organizations/repositoryIdLookupService';
	import {
		PatchEventsService,
		PATCH_EVENTS_SERVICE
	} from '@gitbutler/shared/patchEvents/patchEventsService';
	import {
		PatchCommitService,
		PATCH_COMMIT_SERVICE
	} from '@gitbutler/shared/patches/patchCommitService';
	import {
		PatchIdableService,
		PATCH_IDABLE_SERVICE
	} from '@gitbutler/shared/patches/patchIdableService';
	import { APP_STATE } from '@gitbutler/shared/redux/store.svelte';
	import { RulesService, RULES_SERVICE } from '@gitbutler/shared/rules/rulesService';
	import {
		NotificationSettingsService,
		NOTIFICATION_SETTINGS_SERVICE
	} from '@gitbutler/shared/settings/notificationSettingsService';
	import { UploadsService, UPLOADS_SERVICE } from '@gitbutler/shared/uploads/uploadsService';
	import {
		UserService as NewUserService,
		USER_SERVICE as NEW_USER_SERVICE
	} from '@gitbutler/shared/users/userService';
	import { ChipToastContainer } from '@gitbutler/ui';
	import { setExternalLinkService } from '@gitbutler/ui/utils/externalLinkService';
	import { type Snippet } from 'svelte';
	import { env } from '$env/dynamic/public';

	const CHAT_NOTFICATION_SOUND = '/sounds/pop.mp3';

	interface Props {
		children: Snippet;
	}

	const { children }: Props = $props();

	const authService = inject(AUTH_SERVICE);

	const httpClient = new HttpClient(window.fetch, env.PUBLIC_APP_HOST, authService.tokenReadable);
	provide(HTTP_CLIENT, httpClient);

	const aiService = new ButlerAIClient(httpClient);
	provide(BUTLER_AI_CLIENT, aiService);

	const userService = new UserService(httpClient);
	provide(USER_SERVICE, userService);

	const webState = new WebState();
	provide(APP_STATE, webState);
	provide(WEB_STATE, webState);
	const feedService = new FeedService(httpClient, webState.appDispatch);
	provide(FEED_SERVICE, feedService);
	const organizationService = new OrganizationService(httpClient, webState.appDispatch);
	provide(ORGANIZATION_SERVICE, organizationService);
	const projectService = new ProjectService(httpClient, webState.appDispatch);
	provide(PROJECT_SERVICE, projectService);
	const newUserService = new NewUserService(httpClient, webState.appDispatch);
	provide(NEW_USER_SERVICE, newUserService);
	const branchService = new BranchService(httpClient, webState.appDispatch);
	provide(BRANCH_SERVICE, branchService);
	const patchService = new PatchCommitService(httpClient, webState.appDispatch);
	provide(PATCH_COMMIT_SERVICE, patchService);
	const patchEventsService = new PatchEventsService(
		httpClient,
		webState,
		webState.appDispatch,
		authService.tokenReadable,
		patchService,
		env.PUBLIC_APP_HOST
	);
	const patchIdableService = new PatchIdableService(httpClient, webState.appDispatch);
	provide(PATCH_IDABLE_SERVICE, patchIdableService);

	const user = $derived(userService.user);

	$effect(() => {
		if ($user) {
			patchEventsService.setUserId($user.id);
			patchEventsService.setChatSoundUrl(CHAT_NOTFICATION_SOUND);
		}
	});

	provide(PATCH_EVENTS_SERVICE, patchEventsService);
	const chatChannelService = new ChatChannelsService(httpClient, webState.appDispatch);
	provide(CHAT_CHANNELS_SERVICE, chatChannelService);
	const repositoryIdLookupService = new RepositoryIdLookupService(httpClient, webState.appDispatch);
	provide(REPOSITORY_ID_LOOKUP_SERVICE, repositoryIdLookupService);
	const latestBranchLookupService = new LatestBranchLookupService(httpClient, webState.appDispatch);
	provide(LATEST_BRANCH_LOOKUP_SERVICE, latestBranchLookupService);
	const notificationSettingsService = new NotificationSettingsService(
		httpClient,
		webState.appDispatch
	);
	provide(NOTIFICATION_SETTINGS_SERVICE, notificationSettingsService);
	setExternalLinkService({
		open: (href) => {
			location.href = href;
		}
	});

	const sshKeyService = new SshKeyService(httpClient);
	provide(SSH_KEY_SERVICE, sshKeyService);
	const uploadsService = new UploadsService(httpClient);
	provide(UPLOADS_SERVICE, uploadsService);

	const ownerService = new OwnerService(httpClient);
	provide(OWNER_SERVICE, ownerService);

	const rulesService = new RulesService(httpClient, webState.appDispatch);
	provide(RULES_SERVICE, rulesService);

	const isCommitPage = $derived(page.url.pathname.includes('/commit/'));
</script>

<div class="app">
	{#if !isCommitPage}
		<Navigation />
	{/if}

	<main>
		{@render children?.()}
	</main>
	<Footer />
</div>
<ChipToastContainer />

<style lang="postcss">
	.app {
		--layout-side-paddings: 80px;
		container-type: inline-size;

		display: flex;
		flex-direction: column;
		max-width: calc(1440px + var(--layout-side-paddings) * 2);
		min-height: 100vh;
		margin: 0 auto;
		padding: 24px var(--layout-side-paddings);

		@media (--desktop-small-viewport) {
			--layout-side-paddings: 40px;
		}

		@media (--mobile-viewport) {
			--layout-side-paddings: 16px;
			padding: var(--layout-side-paddings);
		}
	}

	main {
		display: flex;
		flex: 1;
		flex-direction: column;
		width: 100%;
		margin: 0 auto;
	}
</style>
