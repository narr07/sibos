import { spawn } from "node:child_process";
import { createServer } from "node:net";

// Port khusus SIBOS agar tidak bentrok dengan dev server proyek lain (biasanya 3000).
const DEFAULT_PORT = 3210;

/** Port dianggap bebas hanya bila bebas di IPv4 dan IPv6 (server lain bisa hanya listen di ::1). */
function isPortFreeOn(port: number, host: string): Promise<boolean> {
	return new Promise((resolve) => {
		const server = createServer();
		server.once("error", (err: NodeJS.ErrnoException) => {
			// Host IPv6 yang tidak tersedia di mesin ini bukan tanda port terpakai.
			resolve(err.code === "EADDRNOTAVAIL" || err.code === "EAFNOSUPPORT");
		});
		server.once("listening", () => {
			server.close(() => resolve(true));
		});
		server.listen(port, host);
	});
}

async function isPortAvailable(port: number): Promise<boolean> {
	for (const host of ["0.0.0.0", "127.0.0.1", "::", "::1"]) {
		if (!(await isPortFreeOn(port, host))) return false;
	}
	return true;
}

async function findAvailablePort(startPort: number): Promise<number> {
	for (let port = startPort; port < startPort + 100; port++) {
		if (await isPortAvailable(port)) {
			return port;
		}
	}
	throw new Error(`Tidak ada port kosong di rentang ${startPort}-${startPort + 99}`);
}

async function main(): Promise<void> {
	const port = await findAvailablePort(DEFAULT_PORT);

	if (port !== DEFAULT_PORT) {
		console.log(`Port ${DEFAULT_PORT} terpakai, memakai port ${port}`);
	}

	const tauriConfig = JSON.stringify({
		build: {
			// 127.0.0.1, bukan localhost: localhost bisa jatuh ke ::1 milik server lain.
			devUrl: `http://127.0.0.1:${port}`,
			beforeDevCommand: `bun run dev --port ${port} --host 127.0.0.1`
		}
	});

	const tauri = spawn("bun", ["run", "tauri", "dev", "--config", tauriConfig], {
		stdio: "inherit",
		cwd: process.cwd()
	});

	tauri.on("close", (code) => {
		process.exit(code ?? 0);
	});
}

main();
