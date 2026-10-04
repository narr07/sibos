//! Utilitas kecil tanpa dependensi tambahan.

/// Waktu lokal WIB (UTC+7): (tahun, bulan, tanggal, jam, menit, detik).
pub fn now_wib() -> (i32, u32, u32, u32, u32, u32) {
	let secs = std::time::SystemTime::now()
		.duration_since(std::time::UNIX_EPOCH)
		.map(|d| d.as_secs() as i64)
		.unwrap_or(0)
		+ 7 * 3600;
	let days = secs.div_euclid(86_400);
	let rem = secs.rem_euclid(86_400);
	let (y, m, d) = civil_from_days(days);
	(y, m, d, (rem / 3600) as u32, ((rem % 3600) / 60) as u32, (rem % 60) as u32)
}

/// Konversi jumlah hari sejak 1970-01-01 ke tanggal sipil (algoritma Howard Hinnant).
pub fn civil_from_days(days: i64) -> (i32, u32, u32) {
	let z = days + 719_468;
	let era = z.div_euclid(146_097);
	let doe = z - era * 146_097;
	let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
	let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
	let mp = (5 * doy + 2) / 153;
	let day = doy - (153 * mp + 2) / 5 + 1;
	let month = if mp < 10 { mp + 3 } else { mp - 9 };
	let year = yoe + era * 400 + i64::from(month <= 2);
	(year as i32, month as u32, day as u32)
}

/// "20261004-153012" untuk nama file.
pub fn timestamp() -> String {
	let (y, m, d, hh, mm, ss) = now_wib();
	format!("{y:04}{m:02}{d:02}-{hh:02}{mm:02}{ss:02}")
}

/// "2026-10-04 15:30:12".
pub fn datetime_text() -> String {
	let (y, m, d, hh, mm, ss) = now_wib();
	format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}:{ss:02}")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn civil_dates() {
		assert_eq!(civil_from_days(0), (1970, 1, 1));
		assert_eq!(civil_from_days(19_723), (2024, 1, 1));
		assert_eq!(civil_from_days(19_782), (2024, 2, 29));
		assert_eq!(timestamp().len(), 15);
	}
}
