<script lang="ts">
	import { onMount } from 'svelte';
	// Dynamic import to handle Tauri not being available
	let invoke: any;
	let isTauri = false;

	let devices: any[] = [
		{ id: '1', name: 'Poco F5', status: 'Online', connection: 'USB', isPixel: false }
	];
	let loading = true;
	let error = '';
	let selectedDevice = '';
	let packages: string[] = [];

	let chartContainerTotal: HTMLElement;
	let chartContainerMemory: HTMLElement;
	let chartContainerFps: HTMLElement;
	let coreCharts: HTMLElement[] = [];

	let chartTotal: any;
	let chartMemory: any;
	let chartFps: any;
	let dataCpu = Array(40).fill(0);
	let dataMem = Array(40).fill(0);
	let dataFps = Array(40).fill(0);

	let topPackageName = 'Browser';
	let currentFps = 0;
	let currentCpu = 0;
	let currentMem = 0;
	let memStr = '0MB';
	let lastFpsData: any = null;

	let processes = [
		{ name: 'com.android.systemui', pid: '1432', cpu: '12.4', mem: '452 MB', isSys: true },
		{ name: 'com.whatsapp', pid: '8921', cpu: '2.1', mem: '180 MB', isSys: false },
		{ name: 'surfaceflinger', pid: '412', cpu: '15.2', mem: '95 MB', isSys: true }
	];

	onMount(async () => {
		try {
			// Try dynamic import for Tauri
			const tauriApi = await import('@tauri-apps/api/core');
			invoke = tauriApi.invoke;
			isTauri = true;
		} catch (e) {
			console.warn('Tauri not available, using mock mode');
			isTauri = false;
		}

		await loadDevices();

		// ApexCharts init
		const ApexChartsModule = await import('apexcharts');
		const ApexCharts = ApexChartsModule.default;

		const genData = (count: number, min: number, max: number) =>
			Array.from({ length: count }, () => Math.floor(Math.random() * (max - min + 1)) + min);

		const getBaseChartConfig = (color: string, height: number) => ({
			chart: {
				type: 'area',
				sparkline: { enabled: true },
				animations: { enabled: false },
				height: height
			},
			stroke: { curve: 'stepline', width: 1.5 },
			fill: { type: 'solid', opacity: 0.2 },
			colors: [color],
			tooltip: { fixed: { enabled: false }, marker: { show: false } }
		});

		dataCpu = genData(40, 20, 40);
		dataMem = genData(40, 50, 70);
		dataFps = genData(40, 0, 10);

		chartTotal = new ApexCharts(chartContainerTotal, {
			...getBaseChartConfig('#4ADE80', 60),
			series: [{ data: dataCpu }]
		});
		chartTotal.render();

		chartMemory = new ApexCharts(chartContainerMemory, {
			...getBaseChartConfig('#A78BFA', 60),
			series: [{ data: dataMem }]
		});
		chartMemory.render();

		chartFps = new ApexCharts(chartContainerFps, {
			...getBaseChartConfig('#F97316', 60),
			series: [{ data: dataFps }]
		});
		chartFps.render();

		for (let i = 0; i < 8; i++) {
			if (coreCharts[i]) {
				new ApexCharts(coreCharts[i], {
					...getBaseChartConfig('#4ADE80', 35),
					stroke: { curve: 'stepline', width: 1 },
					series: [{ data: genData(25, 0, 100) }]
				}).render();
			}
		}

		setInterval(async () => {
			if (isTauri && invoke && selectedDevice) {
				try {
					const topPkg = await invoke('get_top_package', { serial: selectedDevice });
					if (topPkg && topPkg.name) {
						topPackageName = topPkg.name;
					}

					const perf = await invoke('get_performance_profile', { serial: selectedDevice });
					
					if (perf.memory && perf.memory.Ok) {
						const memInfo = perf.memory.Ok;
						currentMem = Math.round(((memInfo.total - memInfo.available) / memInfo.total) * 100);
						memStr = `${Math.round((memInfo.total - memInfo.available) / 1024)}MB`;
						dataMem.push(currentMem);
						dataMem.shift();
						chartMemory.updateSeries([{ data: dataMem }]);
					}

					if (perf.cpu && perf.cpu.Ok && perf.cpu.Ok.length > 0) {
						currentCpu = Math.round(perf.cpu.Ok[0].usage);
						dataCpu.push(currentCpu);
						dataCpu.shift();
						chartTotal.updateSeries([{ data: dataCpu }]);
					}

					if (perf.fps && perf.fps.Ok) {
						const fpsData = perf.fps.Ok;
						if (fpsData.flips !== null && lastFpsData !== null && fpsData.flips > lastFpsData.flips) {
							const deltaFlips = fpsData.flips - lastFpsData.flips;
							const deltaTime = fpsData.timestamp_ms - lastFpsData.timestamp_ms;
							if (deltaTime > 0) {
								currentFps = Math.round((deltaFlips * 1000) / deltaTime);
							}
						} else if (lastFpsData !== null && fpsData.flips === lastFpsData.flips) {
							currentFps = 0; // Screen is static
						}
						lastFpsData = fpsData;
						
						dataFps.push(currentFps);
						dataFps.shift();
						chartFps.updateSeries([{ data: dataFps }]);
					}
				} catch (e) {
					console.error("Polling error", e);
				}
			}
		}, 1000);
	});

	async function loadDevices() {
		loading = true;
		error = '';
		try {
			if (isTauri && invoke) {
				const rustDevices = await invoke('get_devices');
				if (rustDevices && rustDevices.length > 0) {
					devices = rustDevices.map((d: any) => ({
						id: d.serial,
						name: d.model || d.serial,
						status: 'Online',
						connection: 'USB',
						isPixel: false
					}));
				}
			}
			if (devices.length > 0) {
				selectedDevice = devices[0].id;
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
				// Convert logic if needed. Mock processes stay for now.
			} else {
				packages = ['com.example.app1', 'com.example.app2'];
			}
		} catch (e) {
			error = String(e);
		}
	}
</script>

<main class="flex flex-1 flex-col py-4 pr-4 pl-0 lg:py-6 lg:pr-6 lg:pl-2">
	<div
		class="flex flex-1 flex-col overflow-y-auto rounded-[32px] bg-surface-container-low p-6 lg:p-10 shadow-lg relative"
	>
		<header class="mb-8 flex justify-between items-center">
			<h2 class="text-2xl font-bold tracking-tight text-on-surface flex items-center gap-4">
				KIRA
				{#if !isTauri}
					<span
						class="text-xs bg-error text-on-error px-3 py-1 rounded-full font-medium tracking-normal border border-error/30"
						>MOCK MODE</span
					>
				{/if}
			</h2>
			<button
				on:click={() => {
					const theme =
						document.documentElement.getAttribute('data-theme') === 'light' ? 'dark' : 'light';
					document.documentElement.setAttribute('data-theme', theme);
					window.dispatchEvent(new Event('resize'));
				}}
				class="text-xs bg-surface-variant px-4 py-2 rounded-full text-on-surface font-medium transition-colors hover:brightness-110"
			>
				<span class="material-symbols-outlined text-[16px] align-middle mr-1">palette</span> Toggle Theme
			</button>
		</header>

		{#if devices.length > 0}
			<!-- Current Active Device Banner -->
			<section
				class="relative mb-8 flex flex-col rounded-[28px] bg-surface-container p-8 shadow-sm"
			>
				<div class="mb-6">
					<h1 class="text-[28px] font-bold tracking-tight text-on-surface leading-tight">
						{devices[0].name}
					</h1>
					<p class="mt-2 text-sm text-on-surface-variant font-medium">
						Device connected via {devices[0].connection}
					</p>
				</div>
				<div class="z-10 flex gap-4">
					<button
						class="flex items-center gap-2 rounded-2xl bg-primary px-6 py-3 text-sm font-medium text-on-primary transition-all hover:brightness-110 shadow-sm"
					>
						<span class="material-symbols-outlined text-[18px]">restart_alt</span> Reboot
					</button>
					<button
						class="flex items-center gap-2 rounded-2xl px-6 py-3 text-sm font-medium text-on-surface transition-all hover:bg-surface-variant border border-outline"
					>
						<span class="material-symbols-outlined text-[18px]">terminal</span> Shell
					</button>
				</div>
			</section>
		{/if}

		<section id="device-list" class="flex flex-col gap-3 mb-12">
			{#each devices as d}
				<div
					class="group flex items-center justify-between rounded-[24px] bg-surface-container p-5 shadow-sm transition-all hover:bg-surface-container-high"
				>
					<div class="flex items-center gap-5">
						<div
							class="flex h-14 w-14 items-center justify-center rounded-[16px] bg-surface-container-high text-on-surface transition-colors group-hover:bg-surface-container-highest"
						>
							<span class="material-symbols-outlined text-[28px] opacity-80"
								>{d.isPixel ? 'smartphone' : 'phone_android'}</span
							>
						</div>
						<div class="flex flex-col gap-1">
							<h3 class="text-base font-semibold text-on-surface tracking-wide">{d.name}</h3>
							<div class="flex items-center gap-2 text-[13px] text-on-surface-variant font-medium">
								<span
									class={d.status === 'Fastboot'
										? 'text-secondary'
										: d.status === 'Offline'
											? 'text-error'
											: 'text-primary'}>{d.status}</span
								><span class="opacity-50">•</span><span>{d.connection}</span>
							</div>
						</div>
					</div>
				</div>
			{/each}
		</section>

		<section class="mb-10">
			<div class="flex flex-col rounded-[24px] bg-surface-container p-6 shadow-sm">
				<header class="mb-6 flex items-center justify-between">
					<h3 class="text-lg font-bold tracking-tight text-on-surface flex items-center gap-3">
						<div
							class="flex h-10 w-10 items-center justify-center rounded-[14px] bg-secondary-container text-on-secondary-container"
						>
							<span class="material-symbols-outlined text-[20px]">memory</span>
						</div>
						Process Manager
					</h3>
					<div class="relative">
						<span
							class="material-symbols-outlined absolute left-4 top-1/2 -translate-y-1/2 text-on-surface-variant text-[18px]"
							>search</span
						>
						<input
							type="text"
							placeholder="Search processes..."
							class="bg-surface-container-high rounded-full pl-10 pr-4 py-2.5 text-sm text-on-surface focus:outline-none focus:ring-2 focus:ring-primary/50 transition-all w-64"
						/>
					</div>
				</header>

				<div
					class="flex items-center justify-between px-5 py-2 text-xs font-semibold uppercase tracking-wider text-on-surface-variant mb-2"
				>
					<div class="w-1/3">Package Name</div>
					<div class="flex w-1/2 items-center justify-between pr-10">
						<span class="w-16">PID</span><span class="w-20">CPU</span><span class="w-24 text-right"
							>Memory</span
						>
					</div>
				</div>

				<div class="flex flex-col gap-2">
					{#each processes as proc}
						<div
							class="group flex items-center justify-between rounded-[16px] bg-surface-container-high px-4 py-3 shadow-sm transition-colors hover:bg-surface-container-highest cursor-pointer"
						>
							<div class="flex w-1/3 items-center gap-4 pr-4 pl-1">
								<div
									class="flex h-9 w-9 shrink-0 items-center justify-center rounded-[12px] {proc.isSys
										? 'bg-tertiary-container text-on-tertiary-container'
										: 'bg-surface-variant text-on-surface'}"
								>
									<span class="material-symbols-outlined text-[18px]"
										>{proc.isSys ? 'settings_applications' : 'apps'}</span
									>
								</div>
								<span class="truncate text-[13px] font-medium text-on-surface">{proc.name}</span>
							</div>
							<div class="flex w-1/2 items-center justify-between text-[13px]">
								<span class="w-16 text-on-surface-variant">{proc.pid}</span>
								<span
									class="w-20 {parseFloat(proc.cpu) > 10
										? 'text-error font-semibold'
										: 'text-on-surface-variant'}">{proc.cpu}%</span
								>
								<span class="w-24 text-right text-on-surface-variant">{proc.mem}</span>
							</div>
						</div>
					{/each}
				</div>
			</div>
		</section>

		<section class="mb-10">
			<div class="flex flex-col rounded-[24px] bg-surface-container p-6 shadow-sm">
				<header class="mb-6 flex items-center justify-between">
					<h3 class="text-lg font-bold tracking-tight text-on-surface flex items-center gap-3">
						<div
							class="flex h-10 w-10 items-center justify-center rounded-[14px] bg-primary-container text-on-primary-container"
						>
							<span class="material-symbols-outlined text-[20px]">monitoring</span>
						</div>
						Performance Profiler
					</h3>
					<div class="flex items-center gap-2 rounded-full bg-surface-container-high px-4 py-2">
						<span class="h-2 w-2 rounded-full bg-primary animate-pulse"></span>
						<span class="text-xs font-medium text-on-surface">Uptime 0:00:15:56</span>
					</div>
				</header>

				<div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
					<div
						class="rounded-[20px] bg-surface-container-high p-5 transition-colors hover:bg-surface-container-highest"
					>
						<div class="flex justify-between items-center mb-1">
							<span class="text-xs font-medium text-on-surface-variant">CPU Overall</span>
							<span class="text-sm font-bold text-primary">{currentCpu}%</span>
						</div>
						<div bind:this={chartContainerTotal} class="h-[60px] w-full"></div>
					</div>

					<div
						class="rounded-[20px] bg-surface-container-high p-5 transition-colors hover:bg-surface-container-highest"
					>
						<div class="flex justify-between items-center mb-1">
							<span class="text-xs font-medium text-on-surface-variant">Memory ({currentMem}%)</span>
							<span class="text-sm font-bold text-tertiary">{memStr}</span>
						</div>
						<div bind:this={chartContainerMemory} class="h-[60px] w-full"></div>
					</div>

					<div
						class="rounded-[20px] bg-surface-container-high p-5 transition-colors hover:bg-surface-container-highest"
					>
						<div class="flex justify-between items-center mb-1">
							<span class="text-xs font-medium text-on-surface-variant">FPS {topPackageName}</span>
							<span class="text-sm font-bold text-secondary">{currentFps > 0 ? currentFps : '-'}</span>
						</div>
						<div bind:this={chartContainerFps} class="h-[60px] w-full"></div>
					</div>
				</div>

				<div class="grid grid-cols-2 lg:grid-cols-4 gap-4 mt-4">
					{#each Array(8) as _, i}
						<div
							class="rounded-[16px] bg-surface-container-high p-4 transition-colors hover:bg-surface-container-highest"
						>
							<div class="flex justify-between items-center mb-2">
								<span class="text-[11px] font-medium text-on-surface-variant"
									>CPU{i} <span class="ml-1">1708MHz</span></span
								>
								<span class="text-[12px] font-bold text-primary">~</span>
							</div>
							<div bind:this={coreCharts[i]} class="h-[35px] w-full"></div>
						</div>
					{/each}
				</div>
			</div>
		</section>
	</div>
</main>
