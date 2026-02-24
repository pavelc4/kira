<script lang="ts">
	import { onMount } from 'svelte';

	// Dynamic import to handle Tauri not being available
	let invoke: any;
	let isTauri = false;

	let devices: any[] = [];
	let loading = true;
	let error = '';
	let selectedDevice = '';
	let packages: string[] = [];

	onMount(async () => {
		// Check if Tauri is available
		try {
			const tauriApi = await import('@tauri-apps/api/core');
			invoke = tauriApi.invoke;
			isTauri = true;
		} catch (e) {
			console.warn('Tauri not available, using mock mode');
			isTauri = false;
		}

		await loadDevices();
	});

	async function loadDevices() {
		loading = true;
		error = '';
		try {
			if (isTauri && invoke) {
				devices = await invoke('get_devices');
			} else {
				// Mock data for browser testing
				devices = [{ serial: 'mock_device_001', model: 'Mock Android Device' }];
			}
			if (devices.length > 0) {
				selectedDevice = devices[0].serial;
				await loadPackages();
			}
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	async function loadPackages() {
		if (!selectedDevice) return;
		try {
			if (isTauri && invoke) {
				packages = await invoke('list_packages', {
					serial: selectedDevice,
					filter: 'thirdparty'
				});
			} else {
				// Mock data
				packages = ['com.example.app1', 'com.example.app2', 'com.paget96.batteryguru'];
			}
		} catch (e) {
			error = String(e);
		}
	}

	async function uninstallPackage(pkg: string) {
		try {
			if (isTauri && invoke) {
				const result = await invoke('uninstall_package', {
					serial: selectedDevice,
					packageName: pkg
				});
				alert(JSON.stringify(result));
			} else {
				alert(`[MOCK] Would uninstall: ${pkg}`);
			}
			await loadPackages();
		} catch (e) {
			alert('Error: ' + e);
		}
	}

	async function checkRoot() {
		try {
			if (isTauri && invoke) {
				const isRooted = await invoke('check_root', { serial: selectedDevice });
				alert('Root: ' + isRooted);
			} else {
				alert('[MOCK] Device is rooted: false');
			}
		} catch (e) {
			alert('Error: ' + e);
		}
	}
</script>

<main>
	<h1>Kira - Android Device Manager</h1>

	{#if !isTauri}
		<div class="warning">⚠️ Running in browser mode (Tauri not available)</div>
	{/if}

	{#if loading}
		<p>Loading...</p>
	{:else if error}
		<p class="error">Error: {error}</p>
	{:else}
		<div class="device-selector">
			<label>
				Select Device:
				<select bind:value={selectedDevice} on:change={loadPackages}>
					{#each devices as device}
						<option value={device.serial}>
							{device.model || device.serial}
						</option>
					{/each}
				</select>
			</label>
		</div>

		{#if selectedDevice}
			<div class="actions">
				<button on:click={checkRoot}>Check Root</button>
				<button on:click={loadPackages}>Refresh Apps</button>
			</div>

			<h2>Third-party Apps ({packages.length})</h2>
			<div class="app-list">
				{#each packages as pkg}
					<div class="app-item">
						<span>{pkg}</span>
						<button on:click={() => uninstallPackage(pkg)}>Uninstall</button>
					</div>
				{/each}
			</div>
		{/if}
	{/if}
</main>

<style>
	main {
		padding: 20px;
		max-width: 800px;
		margin: 0 auto;
	}

	.warning {
		background: #fff3cd;
		border: 1px solid #ffc107;
		padding: 10px;
		margin-bottom: 20px;
		border-radius: 4px;
	}

	.error {
		color: red;
	}

	.device-selector {
		margin-bottom: 20px;
	}

	select {
		padding: 8px;
		font-size: 16px;
		margin-left: 10px;
	}

	.actions {
		margin-bottom: 20px;
	}

	button {
		padding: 8px 16px;
		margin-right: 10px;
		cursor: pointer;
	}

	.app-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.app-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 10px;
		border: 1px solid #ccc;
		border-radius: 4px;
	}
</style>
