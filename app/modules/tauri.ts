import * as tauriApp from "@tauri-apps/api/app";
import * as tauriCore from "@tauri-apps/api/core";
import * as tauriDialog from "@tauri-apps/plugin-dialog";
import * as tauriOpener from "@tauri-apps/plugin-opener";
import * as tauriStore from "@tauri-apps/plugin-store";
import { addImports, defineNuxtModule } from "nuxt/kit";

const capitalize = (name: string) => {
	return name.charAt(0).toUpperCase() + name.slice(1);
};

const tauriModules = [
	{ module: tauriApp, prefix: "App", importPath: "@tauri-apps/api/app" },
	{ module: tauriCore, prefix: "Core", importPath: "@tauri-apps/api/core" },
	{ module: tauriDialog, prefix: "Dialog", importPath: "@tauri-apps/plugin-dialog" },
	{ module: tauriOpener, prefix: "Opener", importPath: "@tauri-apps/plugin-opener" },
	{ module: tauriStore, prefix: "Store", importPath: "@tauri-apps/plugin-store" }
];

export default defineNuxtModule<ModuleOptions>({
	meta: {
		name: "nuxt-tauri",
		configKey: "tauri"
	},
	defaults: {
		prefix: "useTauri"
	},
	setup(options) {
		tauriModules.forEach(({ module, prefix, importPath }) => {
			Object.keys(module).filter((name) => name !== "default").forEach((name) => {
				const prefixedName = `${options.prefix}${prefix}` || "";
				const as = prefixedName ? prefixedName + capitalize(name) : name;
				addImports({ from: importPath, name, as });
			});
		});
	}
});
