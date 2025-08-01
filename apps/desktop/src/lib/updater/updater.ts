import { showToast } from '$lib/notifications/toasts';
import { InjectionToken } from '@gitbutler/shared/context';
import { relaunch } from '@tauri-apps/plugin-process';
import { type DownloadEvent, Update } from '@tauri-apps/plugin-updater';
import { get, writable } from 'svelte/store';
import type { PostHogWrapper } from '$lib/analytics/posthog';
import type { Tauri } from '$lib/backend/tauri';
import type { ShortcutService } from '$lib/shortcuts/shortcutService';

export const UPDATER_SERVICE = new InjectionToken<UpdaterService>('UpdaterService');

type UpdateStatus = {
	version?: string;
	releaseNotes?: string;
	status?: InstallStatus | undefined;
};

export type InstallStatus =
	| 'Checking'
	| 'Downloading'
	| 'Downloaded'
	| 'Installing'
	| 'Done'
	| 'Up-to-date'
	| 'Error';

const downloadStatusMap: { [K in DownloadEvent['event']]: InstallStatus } = {
	Started: 'Downloading',
	Progress: 'Downloading',
	Finished: 'Downloaded'
};

export const UPDATE_INTERVAL_MS = 3600000; // Hourly

/**
 * Note that the Tauri API `checkUpdate` hangs indefinitely in dev mode, build
 * a nightly if you want to test the updater manually.
 *
 * export TAURI_SIGNING_PRIVATE_KEY=doesnot
 * export TAURI_SIGNING_PRIVATE_KEY_PASSWORD=matter
 * ./scripts/release.sh --channel nightly --version "0.5.678"
 */
export class UpdaterService {
	readonly disableAutoChecks = writable(false);
	readonly loading = writable(false);
	readonly update = writable<UpdateStatus>({}, () => {
		this.start();
		return () => {
			this.stop();
		};
	});

	private checkForUpdateInterval: ReturnType<typeof setInterval> | undefined;
	private seenVersion: string | undefined;
	private tauriDownload: Update['download'] | undefined;
	private tauriInstall: Update['install'] | undefined;

	unlistenStatus?: () => void;
	unlistenMenu?: () => void;

	constructor(
		private tauri: Tauri,
		private posthog: PostHogWrapper,
		private shortcuts: ShortcutService
	) {}

	private async start() {
		// This shortcut registration is never unsubscribed, but that's likely
		// fine for the time being since the `AppUpdater` can never unmount.
		this.shortcuts.on('update', () => {
			this.checkForUpdate(true);
		});
		this.checkForUpdateInterval = setInterval(
			async () => await this.checkForUpdate(),
			UPDATE_INTERVAL_MS
		);
		this.checkForUpdate();
	}

	private async stop() {
		this.unlistenStatus?.();
		if (this.checkForUpdateInterval !== undefined) {
			clearInterval(this.checkForUpdateInterval);
			this.checkForUpdateInterval = undefined;
		}
	}

	async checkForUpdate(manual = false) {
		if (get(this.disableAutoChecks)) return;

		this.loading.set(true);
		try {
			this.handleUpdate(await this.tauri.checkUpdate(), manual); // In DEV mode this never returns.
		} catch (err: unknown) {
			handleError(err, manual);
		} finally {
			this.loading.set(false);
		}
	}

	private handleUpdate(update: Update | null, manual: boolean) {
		if (update === null) {
			this.update.set({});
			return;
		}
		if (!update.available && manual) {
			this.setStatus('Up-to-date');
		} else if (
			update.available &&
			update.version !== this.seenVersion &&
			update.currentVersion !== '0.0.0' // DEV mode.
		) {
			const { version, body, download, install } = update;
			this.tauriDownload = download.bind(update);
			this.tauriInstall = install.bind(update);
			this.seenVersion = version;
			this.update.set({
				version,
				releaseNotes: body,
				status: undefined
			});
		}
	}

	async downloadAndInstall() {
		this.loading.set(true);
		try {
			await this.download();
			await this.install();
			this.posthog.capture('App Update Successful');
		} catch (error: any) {
			// We expect toast to be shown by error handling in `onUpdaterEvent`
			handleError(error, true);
			this.update.set({ status: 'Error' });
			this.posthog.capture('App Update Install Error', { error });
		} finally {
			this.loading.set(false);
		}
	}

	private async download() {
		if (!this.tauriDownload) {
			throw new Error('Download function not available.');
		}
		this.setStatus('Downloading');
		await this.tauriDownload((progress: DownloadEvent) => {
			this.setStatus(downloadStatusMap[progress.event]);
		});
		this.setStatus('Downloaded');
	}

	private async install() {
		if (!this.tauriInstall) {
			throw new Error('Install function not available.');
		}
		this.setStatus('Installing');
		await this.tauriInstall();
		this.setStatus('Done');
	}

	private setStatus(status: InstallStatus) {
		this.update.update((update) => {
			return { ...update, status };
		});
	}

	async relaunchApp() {
		try {
			await relaunch();
		} catch (err: unknown) {
			handleError(err, true);
		}
	}

	dismiss() {
		this.update.set({});
	}
}

function isOffline(err: any): boolean {
	return (
		typeof err === 'string' &&
		(err.includes('Could not fetch a valid release') ||
			err.includes('error sending request') ||
			err.includes('Network Error'))
	);
}

function handleError(err: any, manual: boolean) {
	if (!manual && isOffline(err)) return;
	console.error(err);
	showToast({
		title: 'App update failed',
		message: `
            Something went wrong while updating the app.

            You can download the latest release from our
            [downloads](https://app.gitbutler.com/downloads) page.
        `,
		error: err,
		style: 'error'
	});
}
