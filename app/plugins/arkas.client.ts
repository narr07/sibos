/** Saat aplikasi dibuka: coba hubungkan ARKAS dengan path dan kunci tersimpan. */
export default defineNuxtPlugin(async () => {
	if (!inTauri()) return;
	const { connect } = useArkas();
	try {
		const result = await connect();
		if (result.state !== "connected") {
			await navigateTo("/setup");
		}
	} catch {
		await navigateTo("/setup");
	}
});
