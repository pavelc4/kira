import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [
		sveltekit(),
		tailwindcss()
	],
	
	// Prevent Vite from clearing screen
	clearScreen: false,
	server: {
		port: 5173,
		strictPort: true,
		hmr: {
			protocol: 'ws',
			host: 'localhost'
		},
		fs: {
			allow: ['..']
		},
		watch: {
			ignored: [
				'**/target/**',
				'**/node_modules/**',
				'**/.git/**'
			]
		}
	},
	
	build: {
		minify: 'esbuild',
		target: 'esnext',
		chunkSizeWarningLimit: 1000
	},
	
	optimizeDeps: {
		include: ['@tauri-apps/api'],
		exclude: []
	}
});
