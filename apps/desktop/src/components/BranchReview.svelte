<script lang="ts">
	import BranchReviewButRequest from '$components/BranchReviewButRequest.svelte';
	import PullRequestCard from '$components/PullRequestCard.svelte';
	import ReviewCreation from '$components/ReviewCreation.svelte';
	import ReviewCreationControls from '$components/ReviewCreationControls.svelte';
	import StackedPullRequestCard from '$components/StackedPullRequestCard.svelte';
	import CanPublishReviewPlugin from '$components/v3/CanPublishReviewPlugin.svelte';
	import { SettingsService } from '$lib/config/appSettingsV2';
	import { syncBrToPr } from '$lib/forge/brToPrSync.svelte';
	import { syncPrToBr } from '$lib/forge/prToBrSync.svelte';
	import { StackService } from '$lib/stacks/stackService.svelte';
	import { UiState } from '$lib/state/uiState.svelte';
	import { TestId } from '$lib/testing/testIds';
	import { getContext } from '@gitbutler/shared/context';
	import { reactive } from '@gitbutler/shared/reactiveUtils.svelte';
	import Button from '@gitbutler/ui/Button.svelte';
	import Modal from '@gitbutler/ui/Modal.svelte';
	import type { Snippet } from 'svelte';

	// TODO: This and the SeriesHeader should have a wholistic refactor to
	// reduce the complexity of the forge related functionality.

	type Props = {
		branchStatus?: Snippet;
		projectId: string;
		stackId?: string;
		branchName: string;
		prNumber?: number;
		reviewId?: string;
	};

	const { branchStatus, projectId, stackId, branchName, prNumber, reviewId }: Props = $props();

	let canPublishReviewPlugin = $state<ReturnType<typeof CanPublishReviewPlugin>>();

	const stackService = getContext(StackService);
	const uiState = getContext(UiState);
	const settingsService = getContext(SettingsService);
	const settingsStore = settingsService.appSettings;
	const commits = $derived(
		stackId ? stackService.commits(projectId, stackId, branchName) : undefined
	);

	const branchConflicted = $derived(
		commits?.current.data?.some((commit) => commit.hasConflicts) || false
	);

	const allowedToPublishBR = $derived(!!canPublishReviewPlugin?.imports.allowedToPublishBR);
	const canPublishPR = $derived(!!canPublishReviewPlugin?.imports.canPublishPR);
	const ctaLabel = $derived(canPublishReviewPlugin?.imports.ctaLabel);
	const branchEmpty = $derived(canPublishReviewPlugin?.imports.branchIsEmpty);

	const disabled = $derived(branchEmpty || branchConflicted);
	const tooltip = $derived(
		branchConflicted ? 'Please resolve the conflicts before creating a PR' : undefined
	);

	let modal = $state<Modal>();
	let confirmCreatePrModal = $state<ReturnType<typeof Modal>>();
	let reviewCreation = $state<ReturnType<typeof ReviewCreation>>();

	syncPrToBr(
		reactive(() => prNumber),
		reactive(() => reviewId)
	);
	syncBrToPr(
		reactive(() => prNumber),
		reactive(() => reviewId)
	);

	const submitDisabled = $derived(reviewCreation ? !reviewCreation.imports.creationEnabled : false);
</script>

<CanPublishReviewPlugin
	bind:this={canPublishReviewPlugin}
	{projectId}
	{stackId}
	{branchName}
	{prNumber}
	{reviewId}
/>

{#if stackId}
	<Modal
		width="small"
		type="warning"
		title="Create Pull Request"
		bind:this={confirmCreatePrModal}
		onSubmit={() => {
			modal?.show();
		}}
	>
		<p class="text-13 text-body helper-text">
			It's strongly recommended to create pull requests starting with the branch at the base of the
			stack.
			<br />
			Do you still want to create this pull request?
		</p>
		{#snippet controls(close)}
			<Button kind="outline" onclick={close}>Cancel</Button>
			<Button style="warning" type="submit">Create Pull Request</Button>
		{/snippet}
	</Modal>

	<Modal bind:this={modal} title="Submit changes for review">
		<ReviewCreation
			bind:this={reviewCreation}
			{projectId}
			{stackId}
			{branchName}
			onClose={() => modal?.close()}
		/>

		{#snippet controls(close)}
			<ReviewCreationControls
				isSubmitting={!!reviewCreation?.imports.isLoading}
				{submitDisabled}
				{canPublishPR}
				onCancel={close}
				onSubmit={async () => {
					await reviewCreation?.createReview();
				}}
			/>
		{/snippet}
	</Modal>
{/if}

<div class="branch-action">
	{#if prNumber || (reviewId && allowedToPublishBR)}
		<div class="status-cards">
			{#if prNumber && stackId}
				<StackedPullRequestCard {projectId} {stackId} {branchName} {prNumber} poll />
			{:else if prNumber}
				<PullRequestCard {branchName} {prNumber} poll />
			{/if}
			{#if reviewId && allowedToPublishBR}
				<BranchReviewButRequest {reviewId} />
			{/if}
		</div>
	{/if}

	{#if branchStatus}
		{@render branchStatus()}
	{/if}

	{#if canPublishPR}
		<Button
			testId={TestId.CreateReviewButton}
			onclick={() => {
				if ($settingsStore?.featureFlags.v3) {
					uiState.project(projectId).exclusiveAction.set({
						type: 'create-pr',
						stackId,
						branchName
					});
				} else {
					modal?.show();
				}
			}}
			kind="outline"
			{disabled}
			{tooltip}
		>
			{ctaLabel}
		</Button>
	{/if}
</div>

<style lang="postcss">
	.branch-action {
		display: flex;
		flex-direction: column;
		width: 100%;
		gap: 14px;
	}

	.status-cards {
		display: flex;
		flex-direction: column;
		gap: 8px;

		& :global(.review-card) {
			display: flex;
			position: relative;
			flex-direction: column;
			padding: 14px;
			gap: 12px;
			border: 1px solid var(--clr-border-2);
			border-radius: var(--radius-m);
		}
	}
</style>
