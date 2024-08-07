<script lang="ts">
	import BranchItem from './BranchItem.svelte';
	import BranchesHeader from './BranchesHeader.svelte';
	import noBranchesSvg from '$lib/assets/empty-state/no-branches.svg?raw';
	import { getBranchServiceStore } from '$lib/branches/service';
	import FilterButton from '$lib/components/FilterBranchesButton.svelte';
	import { getGitHost } from '$lib/gitHost/interface/gitHost';
	import { persisted } from '$lib/persisted/persisted';
	import ScrollableContainer from '$lib/shared/ScrollableContainer.svelte';
	import TextBox from '$lib/shared/TextBox.svelte';
	import { createEventDispatcher } from 'svelte';
	import { derived, readable, writable } from 'svelte/store';
	import type { CombinedBranch } from '$lib/branches/types';

	const dispatch = createEventDispatcher<{ scrollbarDragging: boolean }>();

	export let projectId: string;
	export const textFilter = writable<string | undefined>(undefined);

	const branchService = getBranchServiceStore();
	const gitHost = getGitHost();

	// let contextMenu: ContextMenuActions;
	let includePrs = persisted(true, 'includePrs_' + projectId);
	let includeRemote = persisted(true, 'includeRemote_' + projectId);
	let hideBots = persisted(false, 'hideBots_' + projectId);
	let hideInactive = persisted(false, 'hideInactive_' + projectId);

	let filtersActive = derived(
		[includePrs, includeRemote, hideBots, hideInactive],
		([prs, remote, bots, inactive]) => {
			return !prs || !remote || bots || inactive;
		}
	);

	$: branches = $branchService?.branches || readable([]);
	$: filteredBranches = branches
		? derived(
				[branches, textFilter, includePrs, includeRemote, hideBots, hideInactive],
				([branches, textFilter, includePrs, includeRemote, hideBots, hideInactive]) => {
					const filteredByType = filterByType(branches, {
						includePrs,
						includeRemote,
						hideBots
					});
					const filteredBySearch = filterByText(filteredByType, textFilter);
					return hideInactive ? filterInactive(filteredBySearch) : filteredBySearch;
				}
			)
		: readable([]);

	let viewport: HTMLDivElement;
	let contents: HTMLElement;

	function filterByType(
		branches: CombinedBranch[],
		params: {
			includePrs: boolean;
			includeRemote: boolean;
			hideBots: boolean;
		}
	): CombinedBranch[] {
		return branches.filter((b) => {
			if (params.includePrs && b.pr) {
				return !params.hideBots || !b.pr.author?.isBot;
			} else {
				if (b.pr) return false;
			}

			if (params.includeRemote && b.branch) return true;
			return false;
		});
	}

	function filterByText(branches: CombinedBranch[], search: string | undefined) {
		if (search === undefined) return branches;

		return branches.filter((b) => searchMatchesAnIdentifier(search, b.searchableIdentifiers));
	}

	function searchMatchesAnIdentifier(search: string, identifiers: string[]) {
		for (const identifier of identifiers) {
			if (identifier.includes(search.toLowerCase())) return true;
		}

		return false;
	}

	function filterInactive(branches: CombinedBranch[]) {
		const currentTs = new Date().getTime();
		return branches.filter((b) => {
			if (!b.modifiedAt) return true; // Keep things that don't have a modified time

			const modifiedAt = b.modifiedAt?.getTime();
			const ms = currentTs - modifiedAt;
			return ms < 14 * 86400 * 1000;
		});
	}
</script>

<div class="branch-list">
	<BranchesHeader
		totalBranchCount={$branches.length}
		filteredBranchCount={$filteredBranches?.length}
		filtersActive={$filtersActive}
	>
		{#snippet filterButton()}
			<FilterButton
				{filtersActive}
				{includePrs}
				{includeRemote}
				{hideBots}
				{hideInactive}
				showPrCheckbox={!!$gitHost}
				on:action
			/>
		{/snippet}
	</BranchesHeader>
	{#if $branches.length > 0}
		<ScrollableContainer
			bind:viewport
			showBorderWhenScrolled
			on:dragging={(e) => dispatch('scrollbarDragging', e.detail)}
			fillViewport={$filteredBranches.length === 0}
		>
			<div class="scroll-container">
				<TextBox icon="search" placeholder="Search" on:input={(e) => textFilter.set(e.detail)} />

				{#if $filteredBranches.length > 0}
					<div bind:this={contents} class="content">
						{#each $filteredBranches as branch}
							<BranchItem {projectId} {branch} />
						{/each}
					</div>
				{:else}
					<div class="branch-list__empty-state">
						<div class="branch-list__empty-state__image">
							{@html noBranchesSvg}
						</div>
						<span class="branch-list__empty-state__caption text-base-body-14 text-semibold"
							>No branches match your filter</span
						>
					</div>
				{/if}
			</div>
		</ScrollableContainer>
	{:else}
		<div class="branch-list__empty-state">
			<div class="branch-list__empty-state__image">
				{@html noBranchesSvg}
			</div>
			<span class="branch-list__empty-state__caption text-base-body-14 text-semibold"
				>You have no branches</span
			>
		</div>
	{/if}
</div>

<style lang="postcss">
	.scroll-container {
		display: flex;
		flex-direction: column;
		gap: 12px;
		width: 100%;
		height: 100%;
		padding-bottom: 16px;
		padding-left: 14px;
		padding-right: 14px;
	}
	.branch-list {
		flex: 1;
		position: relative;
		overflow: hidden;
		display: flex;
		flex-direction: column;
		border-top: 1px solid var(--clr-border-2);
	}
	.content {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		gap: 2px;
	}

	/* EMPTY STATE */
	.branch-list__empty-state {
		flex: 1;
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		gap: 10px;
	}

	.branch-list__empty-state__image {
		width: 130px;
	}

	.branch-list__empty-state__caption {
		color: var(--clr-scale-ntrl-60);
		text-align: center;
		max-width: 160px;
	}
</style>
