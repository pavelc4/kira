<script lang="ts">
</script>

<nav class="flex w-24 flex-col items-center gap-6 py-8">
    <button
        class="flex h-10 w-10 items-center justify-center rounded-full text-on-surface-variant transition-colors hover:bg-surface-variant hover:text-on-surface"
    >
        <span class="material-symbols-outlined">menu</span>
    </button>
    <button
        class="mb-4 flex h-14 w-14 items-center justify-center rounded-[20px] bg-primary-container text-on-primary-container shadow-lg transition-transform hover:brightness-110 active:scale-95"
        on:click={() => alert('Scanning for ADB/Fastboot devices...')}
    >
        <span class="material-symbols-outlined">refresh</span>
    </button>
    <a
        href="/"
        class="group flex flex-col items-center gap-1.5 text-[11px] font-medium text-on-surface text-decoration-none"
    >
        <div
            class="flex h-9 w-16 items-center justify-center rounded-2xl bg-secondary-container text-on-secondary-container transition-colors"
        >
            <span class="material-symbols-outlined text-[22px]" style="font-variation-settings: 'FILL' 1;"
                >devices</span
            >
        </div>
        <span>Devices</span>
    </a>
    <a
        href="/"
        class="group flex flex-col items-center gap-1.5 text-[11px] font-medium text-on-surface-variant hover:text-on-surface text-decoration-none"
    >
        <div
            class="flex h-9 w-16 items-center justify-center rounded-2xl transition-colors group-hover:bg-surface-variant"
        >
            <span class="material-symbols-outlined text-[22px]">terminal</span>
        </div>
        <span>Logcat</span>
    </a>
</nav>