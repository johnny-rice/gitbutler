import { createKeybindingsHandler } from 'tinykeys';

interface KeybindDefinitions {
	[combo: string]: (event: KeyboardEvent) => void;
}

export const shortcuts = {
	global: {
		open_repository: {
			title: 'Add local repository…',
			keys: '$mod+O',
			description: null
		},
		clone_repository: {
			title: 'Clone repository',
			keys: '$mod+Shift+O',
			description: null
		}
	},
	view: {
		switch_theme: {
			title: 'Switch theme',
			keys: '$mod+T',
			description: null
		},
		toggle_sidebar: {
			title: 'Toggle sidebar',
			keys: '$mod+\\',
			description: null
		},
		zoom_in: {
			title: 'Zoom in',
			keys: '$mod+=',
			description: null
		},
		zoom_out: {
			title: 'Zoom out',
			keys: '$mod+-',
			description: null
		},
		reset_zoom: {
			title: 'Reset zoom',
			keys: '$mod+0',
			description: null
		},
		reload_view: {
			title: 'Reload view',
			keys: '$mod+R',
			description: null
		}
	},
	project: {
		project_history: {
			title: 'Project history',
			description: 'Opens the project history view. Revert changes, view commits, and more.',
			keys: '$mod+Shift+H'
		}
	},
	uncommitted_changes: {
		branch: {
			title: 'Branch changes',
			description: 'Create a new branch and commit out of the selected files.',
			keys: '$mod+B'
		}
	}
};

export function createKeybind(keybinds: KeybindDefinitions) {
	const keys: KeybindDefinitions = {
		// Ignore backspace keydown events always
		Backspace: () => {}
	};

	for (const [combo, callback] of Object.entries(keybinds)) {
		keys[combo] = (event: KeyboardEvent) => {
			if (
				event.repeat ||
				event.target instanceof HTMLInputElement ||
				event.target instanceof HTMLTextAreaElement
			) {
				return;
			}

			event.preventDefault();

			callback(event);
		};
	}

	return createKeybindingsHandler(keys);
}
