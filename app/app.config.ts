export default defineAppConfig({
	app: {
		name: "SIBOS",
		tagline: "Pendamping ARKAS untuk SPJ BOSP"
	},
	ui: {
		colors: {
			primary: "emerald",
			neutral: "slate"
		},
		button: {
			slots: {
				base: "cursor-pointer"
			}
		},
		formField: {
			slots: {
				root: "w-full"
			}
		},
		input: {
			slots: {
				root: "w-full"
			}
		},
		navigationMenu: {
			slots: {
				link: "cursor-pointer"
			}
		}
	}
});
