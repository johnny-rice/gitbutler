<script lang="ts">
	import SnapshotAttachment from '$components/SnapshotAttachment.svelte';
	import { createdOnDay } from '$lib/history/history';
	import { MODE_SERVICE } from '$lib/mode/modeService';
	import { toHumanReadableTime } from '$lib/utils/time';
	import { inject } from '@gitbutler/shared/context';
	import { Button, Icon, FileListItem } from '@gitbutler/ui';
	import type { Snapshot, SnapshotDetails } from '$lib/history/types';
	import type iconsJson from '@gitbutler/ui/data/icons.json';

	interface Props {
		entry: Snapshot;
		isWithinRestore?: boolean;
		onRestoreClick: () => void;
		onDiffClick: (filePath: string) => void;
		selectedFile?:
			| {
					entryId: string;
					path: string;
			  }
			| undefined;
		projectId: string;
	}

	const {
		projectId,
		entry,
		isWithinRestore = true,
		selectedFile = undefined,
		onRestoreClick,
		onDiffClick
	}: Props = $props();

	function getShortSha(sha: string | undefined) {
		if (!sha) return '';

		return `${sha.slice(0, 7)}`;
	}

	function createdOnDayAndTime(epoch: number) {
		const date = new Date(epoch);
		return `${createdOnDay(date)}, ${toHumanReadableTime(date)}`;
	}

	function camelToTitleCase(str: string | undefined) {
		if (!str) return '';
		const lowerCaseStr = str.replace(/([a-z])([A-Z])/g, '$1 $2').toLowerCase();
		return lowerCaseStr.charAt(0).toUpperCase() + lowerCaseStr.slice(1);
	}

	function mapOperation(snapshotDetails: SnapshotDetails | undefined): {
		text: string;
		icon?: keyof typeof iconsJson;
		commitMessage?: string;
	} {
		if (!snapshotDetails) return { text: '', icon: 'commit' };

		switch (snapshotDetails.operation) {
			// BRANCH OPERATIONS
			case 'DeleteBranch':
				return {
					text: `Delete branch "${entry.details?.trailers.find((t) => t.key === 'name')?.value}"`,
					icon: 'item-cross'
				};
			case 'ApplyBranch':
				return {
					text: `Apply branch "${entry.details?.trailers.find((t) => t.key === 'name')?.value}"`,
					icon: 'item-tick'
				};
			case 'UnapplyBranch':
				return {
					text: `Unapply branch "${snapshotDetails.trailers.find((t) => t.key === 'name')?.value}"`,
					icon: 'item-dashed'
				};
			case 'UpdateBranchName':
				return {
					text: `Renamed branch "${snapshotDetails.trailers.find((t) => t.key === 'before')?.value}" to "${snapshotDetails.trailers.find((t) => t.key === 'after')?.value}"`,
					icon: 'item-slash'
				};
			case 'CreateBranch':
				return {
					text: `Create branch "${snapshotDetails.trailers.find((t) => t.key === 'name')?.value}"`,
					icon: 'item-plus'
				};
			case 'ReorderBranches':
				return {
					text: `Reorder branches "${snapshotDetails.trailers.find((t) => t.key === 'before')?.value}" and "${snapshotDetails.trailers.find((t) => t.key === 'after')?.value}"`,
					icon: 'item-link'
				};
			case 'SelectDefaultVirtualBranch':
				return {
					text: `Select default virtual branch "${snapshotDetails.trailers.find((t) => t.key === 'after')?.value}"`,
					icon: 'item-dot'
				};
			case 'UpdateBranchRemoteName':
				return {
					text: `Update branch remote name "${snapshotDetails.trailers.find((t) => t.key === 'before')?.value}" to "${snapshotDetails.trailers.find((t) => t.key === 'after')?.value}"`,
					icon: 'item-slash'
				};
			case 'SetBaseBranch':
				return { text: 'Set base branch', icon: 'item-slash' };
			case 'GenericBranchUpdate':
				return { text: 'Generic branch update', icon: 'item-slash' };

			// COMMIT OPERATIONS
			case 'CreateCommit':
				return {
					text: `Create commit ${getShortSha(entry.details?.trailers.find((t) => t.key === 'sha')?.value)}`,
					icon: 'new-commit',
					commitMessage: entry.details?.trailers.find((t) => t.key === 'message')?.value
				};
			case 'UndoCommit':
				return {
					text: `Undo commit ${getShortSha(entry.details?.trailers.find((t) => t.key === 'sha')?.value)}`,
					icon: 'undo-commit',
					commitMessage: entry.details?.trailers.find((t) => t.key === 'message')?.value
				};
			case 'AmendCommit':
				return { text: 'Amend commit', icon: 'amend-commit' };
			case 'SquashCommit':
				return { text: 'Squash commit', icon: 'squash-commit' };
			case 'UpdateCommitMessage':
				return { text: 'Update commit message', icon: 'edit-text' };
			case 'MoveCommit':
				return { text: 'Move commit', icon: 'move-commit' };
			case 'ReorderCommit':
				return { text: 'Reorder commit', icon: 'move-commit' };
			case 'InsertBlankCommit':
				return { text: 'Insert blank commit', icon: 'blank-commit' };
			case 'MoveCommitFile':
				return { text: 'Move commit file', icon: 'move-commit-file-small' };

			// FILE OPERATIONS
			case 'MoveHunk':
				return {
					text: `Move hunk to "${entry.details?.trailers.find((t) => t.key === 'name')?.value}"`,
					icon: 'item-move'
				};
			case 'DiscardLines':
				return { text: 'Discard lines', icon: 'item-cross' };
			case 'DiscardHunk':
				return { text: 'Discard change', icon: 'item-cross' };
			case 'DiscardFile':
				return { text: 'Discard file', icon: 'discard-file-small' };
			case 'FileChanges':
				return { text: 'File changes', icon: 'file-changes-small' };

			// OTHER OPERATIONS
			case 'MergeUpstream':
				return { text: 'Merge upstream', icon: 'merged-pr-small' };
			case 'UpdateWorkspaceBase':
				return { text: 'Update workspace base', icon: 'rebase' };
			case 'EnterEditMode':
				return { text: 'Enter Edit Mode', icon: 'edit-text' };
			case 'RestoreFromSnapshot':
				return { text: 'Revert snapshot' };
			case 'SplitBranch':
				return { text: 'Split branch', icon: 'branch-local' };
			default:
				return { text: snapshotDetails.operation, icon: 'commit' };
		}
	}

	const isRestoreSnapshot = entry.details?.operation === 'RestoreFromSnapshot';
	const error = entry.details?.trailers.find((t) => t.key === 'error')?.value;

	const operation = mapOperation(entry.details);

	const modeService = inject(MODE_SERVICE);
	const mode = $derived(modeService.mode({ projectId }));
</script>

<div
	class="snapshot-card show-restore-on-hover"
	class:restored-snapshot={isRestoreSnapshot || isWithinRestore}
>
	<div class="snapshot-right-container">
		<div class="restore-btn">
			<Button
				size="tag"
				kind="outline"
				tooltip="Restores GitButler and your files to the state before this operation. Revert actions can also be undone."
				onclick={() => {
					onRestoreClick();
				}}
				disabled={mode.current.data?.type !== 'OpenWorkspace'}>Revert</Button
			>
		</div>
		<span class="snapshot-time text-11">
			{toHumanReadableTime(entry.createdAt)}
		</span>
	</div>

	<div class="snapshot-line">
		{#if isRestoreSnapshot}
			<img src="/images/history/restore-icon.svg" alt="" />
		{:else if operation.icon}
			<Icon name={operation.icon} />
		{/if}
	</div>

	<div class="snapshot-content">
		<div class="snapshot-details">
			<h4 class="snapshot-title text-13 text-body text-semibold">
				<span>{operation.text}</span>
				<span class="snapshot-sha text-12 text-body"> • {getShortSha(entry.id)}</span>
			</h4>

			{#if operation.commitMessage}
				<p class="text-12 text-body snapshot-commit-message">
					<span>Message:</span>
					{operation.commitMessage}
				</p>
			{/if}
		</div>

		{#if entry.filesChanged.length > 0 && !isRestoreSnapshot}
			{@const files = entry.filesChanged}
			<SnapshotAttachment
				foldable={entry.filesChanged.length > 2}
				foldedAmount={entry.filesChanged.length}
			>
				<div class="files-attacment">
					{#each files as filePath}
						<FileListItem
							listMode="list"
							{filePath}
							onclick={() => onDiffClick(filePath)}
							selected={selectedFile?.path === filePath && selectedFile?.entryId === entry.id}
							hideBorder={filePath === files[files.length - 1]}
						/>
					{/each}
				</div>
			</SnapshotAttachment>
		{/if}

		{#if isRestoreSnapshot}
			<SnapshotAttachment>
				<div class="restored-attacment">
					<Icon name="commit" />
					<div class="restored-attacment__content">
						<h4 class="text-13 text-semibold">
							{camelToTitleCase(
								entry.details?.trailers.find((t) => t.key === 'restored_operation')?.value
							)}
						</h4>
						<span class="restored-attacment__details text-12">
							{getShortSha(entry.details?.trailers.find((t) => t.key === 'restored_from')?.value)} •
							{createdOnDayAndTime(
								parseInt(
									entry.details?.trailers.find((t) => t.key === 'restored_date')?.value || ''
								)
							)}
						</span>
					</div>
				</div>
			</SnapshotAttachment>
		{/if}
		{#if error}
			<div class="error-text text-12 text-body">
				{error}
			</div>
		{/if}
	</div>
</div>

<style lang="postcss">
	/* SNAPSHOT CARD */
	.snapshot-card {
		display: flex;
		position: relative;
		padding: 10px 14px 8px 14px;
		overflow: hidden;
		gap: 12px;
		background-color: var(--clr-bg-1);
		transition: padding 0.2s;
	}

	.show-restore-on-hover {
		&:hover {
			background-color: var(--clr-bg-1-muted);
			& .restore-btn {
				display: flex;
			}

			& .snapshot-time {
				display: none;
			}
		}
	}

	.snapshot-right-container {
		display: flex;
		justify-content: flex-end;
		width: 60px;
	}

	.restore-btn {
		display: none;
	}

	.snapshot-time {
		margin-top: 2px;
		color: var(--clr-text-2);
		line-height: 1.8;
		text-align: right;
	}

	.snapshot-line {
		display: flex;
		position: relative;
		flex-direction: column;
		align-items: center;
		margin-top: 3px;

		&::after {
			position: absolute;
			top: 24px;
			width: 1px;
			height: calc(100% - 14px);
			min-height: 8px;
			background-color: var(--clr-border-2);
			content: '';
		}
	}

	/* CARD CONTENT */
	.snapshot-content {
		display: flex;
		flex: 1;
		flex-direction: column;
		align-items: flex-start;
		min-height: var(--size-tag);
		overflow: hidden;
		gap: 6px;
	}

	.snapshot-details {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		width: 100%;
		margin-top: 2px;
		margin-bottom: 4px;
		gap: 6px;
	}

	.snapshot-title {
		flex: 1;
	}

	.snapshot-commit-message {
		margin-bottom: 2px;
		color: var(--clr-text-2);

		& span {
			color: var(--clr-text-3);
		}
	}

	.snapshot-sha {
		color: var(--clr-text-3);
		white-space: nowrap;
	}

	/* ATTACHMENT FILES */
	.files-attacment {
		display: flex;
		flex-direction: column;
	}

	/* ATTACHMENT RESTORE */
	.restored-attacment {
		display: flex;
		padding: 12px;
		gap: 8px;
	}

	.restored-attacment__content {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.restored-attacment__details {
		color: var(--clr-text-2);
	}

	/* RESTORED  */
	.restored-snapshot {
		background-color: var(--clr-bg-2);
	}

	/* --- */
	.error-text {
		display: flex;
		width: 100%;
		padding: 6px 10px;
		border-radius: var(--radius-m);
		background-color: var(--clr-theme-err-bg-muted);
		color: var(--clr-scale-err-40);
	}
</style>
