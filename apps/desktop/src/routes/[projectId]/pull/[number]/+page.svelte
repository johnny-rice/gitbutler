<script lang="ts">
	// This page is displayed when:
	// - A pr is found
	// - And it does NOT have a cooresponding vbranch
	// - And it does NOT have a cooresponding remote
	// It may also display details about a cooresponding pr if they exist
	import { page } from '$app/state';
	import FullviewLoading from '$components/FullviewLoading.svelte';
	import PullRequestPreview from '$components/PullRequestPreview.svelte';
	import { DefaultForgeFactory } from '$lib/forge/forgeFactory.svelte';
	import { getContext } from '@gitbutler/shared/context';

	const projectId = page.params.projectId!;
	const forge = getContext(DefaultForgeFactory);
	const forgeListing = $derived(forge.current.listService);
	const prsResult = $derived(forgeListing?.list(projectId));
	const pr = $derived(
		prsResult?.current.data?.find((b) => b.number.toString() === page.params.number)
	);
</script>

<div class="wrapper">
	<div class="inner">
		{#if !pr}
			<FullviewLoading />
		{:else if pr}
			<PullRequestPreview {pr} />
		{:else}
			<p>Branch doesn't seem to exist</p>
		{/if}
	</div>
</div>

<style lang="postcss">
	.wrapper {
		display: flex;
		height: 100%;
		overflow-y: auto;
		overscroll-behavior: none;
	}
	.inner {
		display: flex;
		padding: 16px;
	}
</style>
