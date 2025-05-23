<script lang="ts">
	import FileList from '$components/v3/FileList.svelte';
	import FileListMode from '$components/v3/FileListMode.svelte';
	import emptyFolderSvg from '$lib/assets/empty-state/empty-folder.svg?raw';
	import Badge from '@gitbutler/ui/Badge.svelte';
	import EmptyStatePlaceholder from '@gitbutler/ui/EmptyStatePlaceholder.svelte';
	import { stickyHeader } from '@gitbutler/ui/utils/stickyHeader';
	import type { ConflictEntriesObj } from '$lib/files/conflicts';
	import type { TreeChange } from '$lib/hunks/change';
	import type { SelectionId } from '$lib/selection/key';

	type Props = {
		projectId: string;
		stackId?: string;
		selectionId: SelectionId;
		changes: TreeChange[];
		title: string;
		active?: boolean;
		conflictEntries?: ConflictEntriesObj;
	};

	const { projectId, stackId, selectionId, changes, title, active, conflictEntries }: Props =
		$props();

	let listMode: 'list' | 'tree' = $state('tree');
</script>

<div class="changed-files__header" use:stickyHeader>
	<div class="changed-files__header-left">
		<h4 class="text-14 text-semibold truncate">{title}</h4>
		<Badge>{changes.length}</Badge>
	</div>
	<FileListMode bind:mode={listMode} persist="committed" />
</div>
{#if changes.length > 0}
	<FileList {selectionId} {projectId} {stackId} {changes} {listMode} {active} {conflictEntries} />
{:else}
	<EmptyStatePlaceholder image={emptyFolderSvg} width={180} gap={4}>
		{#snippet caption()}
			No files changed
		{/snippet}
	</EmptyStatePlaceholder>
{/if}

<style>
	.changed-files__header {
		padding: 10px 10px 10px 14px;
		display: flex;
		align-items: center;
		gap: 8px;
		justify-content: space-between;
		background-color: var(--clr-bg-1);
	}

	.changed-files__header-left {
		display: flex;
		align-items: center;
		gap: 6px;
		overflow: hidden;
	}
</style>
