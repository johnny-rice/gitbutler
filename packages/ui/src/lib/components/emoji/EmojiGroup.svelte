<script lang="ts">
	import EmojiButton from '$components/emoji/EmojiButton.svelte';
	import { intersectionObserver } from '$lib/utils/intersectionObserver';
	import type { EmojiGroup, EmojiInfo } from '$components/emoji/utils';

	type Props = {
		group: EmojiGroup;
		handleEmojiClick: (emoji: EmojiInfo) => void;
	};

	let { group, handleEmojiClick }: Props = $props();

	let isInViewport = $state(false);
</script>

<div
	use:intersectionObserver={{
		callback: (entry) => {
			if (entry?.isIntersecting) {
				isInViewport = true;
			}
		},
		options: {
			threshold: 0.5,
			root: null
		}
	}}
	class="emoji-picker__group"
	class:min-height={group.key !== 'recently-used'}
	id="emoji-group-{group.key}"
>
	{#if isInViewport}
		{#each group.emojis as emoji}
			<EmojiButton emoji={emoji.unicode} onclick={() => handleEmojiClick(emoji)} />
		{/each}
	{/if}
</div>

<style lang="postcss">
	.emoji-picker__group {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		grid-auto-rows: 1fr;
		row-gap: 4px;
		align-items: center;
		justify-items: center;
		padding: 8px 6px;

		&:not(:last-child) {
			border-bottom: 1px solid var(--clr-border-3);
		}

		&.min-height {
			min-height: 200px;
		}
	}
</style>
