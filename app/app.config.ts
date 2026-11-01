export default defineAppConfig({
	app: {
		name: "Lajur",
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
		// Sudut siku: komponen yang bawaannya rounded-full (tidak ikut --ui-radius)
		switch: {
			slots: {
				base: "rounded-none",
				thumb: "rounded-none"
			}
		},
		slider: {
			slots: {
				track: "rounded-none",
				range: "rounded-none",
				thumb: "rounded-none"
			}
		},
		progress: {
			slots: {
				base: "rounded-none",
				indicator: "rounded-none"
			}
		},
		avatar: {
			slots: {
				root: "rounded-none"
			}
		},
		chip: {
			slots: {
				base: "rounded-none"
			}
		},
		colorPicker: {
			slots: {
				selectorThumb: "rounded-none",
				trackThumb: "rounded-none"
			}
		},
		fileUpload: {
			slots: {
				fileTrailingButton: "rounded-none"
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
