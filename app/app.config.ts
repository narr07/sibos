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
			},
			// Menu aktif: solid (latar warna utama tema, teks & ikon putih)
			compoundVariants: [{
				color: "primary",
				variant: "pill",
				active: true,
				class: {
					link: "text-inverted before:bg-primary",
					linkLeadingIcon: "text-inverted group-data-[state=open]:text-inverted",
					linkTrailingIcon: "text-inverted"
				}
			}]
		}
	}
});
