/**
 * Build installer SIBOS (.exe NSIS + .msi) yang ditandatangani untuk pembaruan otomatis,
 * lalu (opsional) terbitkan ke GitHub Releases beserta latest.json.
 *
 *   bun run rilis              -> build + siapkan latest.json
 *   bun run rilis --terbitkan  -> build + buat GitHub Release (butuh `gh` yang sudah login)
 *
 * Kunci tanda tangan dibaca dari %USERPROFILE%\.tauri\sibos.key (tidak pernah masuk repo).
 * Catatan rilis diambil dari file CATATAN_RILIS.md (opsional, tidak ikut repo).
 */
import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import process from "node:process";
import { $ } from "bun";

const REPO = "narr07/sibos";
const root = join(import.meta.dir, "..");
const conf = JSON.parse(readFileSync(join(root, "src-tauri", "tauri.conf.json"), "utf8"));
const versi: string = conf.version;
const tag = `v${versi}`;
const terbitkan = process.argv.includes("--terbitkan");

const kunci = join(homedir(), ".tauri", "sibos.key");
if (!existsSync(kunci)) {
	console.error(`Kunci tanda tangan tidak ditemukan: ${kunci}`);
	process.exit(1);
}

console.log(`Build SIBOS ${versi} ...`);
await $`bun run tauri build`.cwd(root).env({
	...process.env,
	TAURI_SIGNING_PRIVATE_KEY: readFileSync(kunci, "utf8"),
	TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ""
});

const bundle = join(root, "src-tauri", "target", "release", "bundle");
const exe = join(bundle, "nsis", `SIBOS_${versi}_x64-setup.exe`);
const msi = join(bundle, "msi", `SIBOS_${versi}_x64_en-US.msi`);
for (const f of [exe, `${exe}.sig`, msi, `${msi}.sig`]) {
	if (!existsSync(f)) {
		console.error(`File hasil build tidak ditemukan: ${f}`);
		process.exit(1);
	}
}

const catatanFile = join(root, "CATATAN_RILIS.md");
const catatan = existsSync(catatanFile) ? readFileSync(catatanFile, "utf8").trim() : `SIBOS ${versi}`;

// Pembaruan otomatis memakai installer NSIS (mode passive, terpasang di Program Files).
const latest = {
	version: versi,
	notes: catatan,
	pub_date: new Date().toISOString(),
	platforms: {
		"windows-x86_64": {
			signature: readFileSync(`${exe}.sig`, "utf8").trim(),
			url: `https://github.com/${REPO}/releases/download/${tag}/SIBOS_${versi}_x64-setup.exe`
		}
	}
};
const latestFile = join(bundle, "latest.json");
writeFileSync(latestFile, `${JSON.stringify(latest, null, 2)}\n`);

console.log("\nHasil build:");
for (const f of [exe, msi, latestFile]) console.log(`  ${f}`);

if (terbitkan) {
	console.log(`\nMenerbitkan ${tag} ke GitHub Releases ...`);
	await $`gh release create ${tag} ${exe} ${`${exe}.sig`} ${msi} ${`${msi}.sig`} ${latestFile} --repo ${REPO} --title ${`SIBOS ${versi}`} --notes ${catatan}`.cwd(root);
	console.log("Selesai. Aplikasi yang sudah terpasang akan menerima notifikasi pembaruan.");
} else {
	console.log("\nBelum diterbitkan. Jalankan `bun run rilis --terbitkan` untuk membuat GitHub Release.");
}
