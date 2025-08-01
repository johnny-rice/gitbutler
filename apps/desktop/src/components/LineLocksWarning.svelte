<script lang="ts">
	import ReduxResult from '$components/ReduxResult.svelte';
	import { getStackName } from '$lib/stacks/stack';
	import { STACK_SERVICE } from '$lib/stacks/stackService.svelte';
	import { TestId } from '$lib/testing/testIds';
	import { inject } from '@gitbutler/shared/context';
	import type { DependencyLock } from '@gitbutler/ui/utils/diffParsing';

	type Props = {
		projectId: string;
		locks: DependencyLock[];
	};

	const { projectId, locks }: Props = $props();

	const stackService = inject(STACK_SERVICE);

	const lockedToStackIds = $derived(locks.map((lock) => lock.stackId));
	const stacksResult = $derived(stackService.stacks(projectId));
</script>

<ReduxResult result={stacksResult.current} {projectId}>
	{#snippet children(stacks)}
		{@const lockedToStacks = stacks.filter((stack) => lockedToStackIds.includes(stack.id))}
		{@const stackNames = lockedToStacks.map(getStackName)}
		<div data-testid={TestId.UnifiedDiffViewLockWarning}>
			{#if stackNames.length > 1}
				<p>This line depends on changes inside the following stacks</p>
				<br />
				<p>{stackNames.join(', ')}</p>
			{:else if stackNames.length === 1}
				<p>This line depends on changes inside <b>'{stackNames[0]}'</b></p>
			{:else}
				<p>This is weird and shouldn't happen</p>
			{/if}
		</div>
	{/snippet}
</ReduxResult>
