//! Permintaan HTTP ke situs SIPLah lewat curl bawaan Windows (tanpa jendela console).

use std::os::windows::process::CommandExt;

use crate::error::{AppError, AppResult};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const CURL: &str = "C:\\Windows\\System32\\curl.exe";
const USER_AGENT: &str = "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

fn run(args: &[&str]) -> AppResult<String> {
	let mut cmd = std::process::Command::new(CURL);
	cmd.creation_flags(CREATE_NO_WINDOW);
	cmd.args([
		// --fail: status HTTP 4xx/5xx jadi error (exit 22), bukan halaman error yang dikira data.
		"--fail",
		"-sS",
		"-L",
		"--max-time",
		"45",
		"-H",
		USER_AGENT,
	]);
	cmd.args(args);
	let output = cmd.output().map_err(|e| AppError::Io(std::io::Error::other(format!("gagal menjalankan curl: {e}"))))?;
	if !output.status.success() {
		let detail = String::from_utf8_lossy(&output.stderr).trim().trim_start_matches("curl: ").to_string();
		return Err(AppError::InvalidInput(curl_error_message(output.status.code(), &detail)));
	}
	Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// GET dan kembalikan isi respons sebagai teks (HTML atau JSON).
pub fn get(url: &str, accept: &str) -> AppResult<String> {
	run(&["-H", &format!("Accept: {accept}"), url])
}

/// POST form (application/x-www-form-urlencoded) dengan header tambahan.
pub fn post_form(url: &str, fields: &[(&str, &str)], headers: &[&str]) -> AppResult<String> {
	let body = fields.iter().map(|(k, v)| format!("{}={}", encode(k), encode(v))).collect::<Vec<_>>().join("&");
	let mut args: Vec<&str> = vec!["-H", "Accept: application/json, text/plain, */*"];
	for h in headers {
		args.extend(["-H", h]);
	}
	// --data-raw: isi tidak pernah dibaca sebagai nama file (beda dengan -d yang menerima "@file").
	args.extend(["--data-raw", &body, url]);
	run(&args)
}

/// Percent-encoding untuk nilai form.
fn encode(s: &str) -> String {
	let mut out = String::with_capacity(s.len());
	for b in s.bytes() {
		match b {
			b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
			_ => out.push_str(&format!("%{b:02X}")),
		}
	}
	out
}

/// Pesan yang bisa dipahami bendahara untuk kode exit curl yang umum.
pub fn curl_error_message(code: Option<i32>, detail: &str) -> String {
	let pesan = match code {
		Some(6) => "Server SIPLah tidak ditemukan. Periksa koneksi internet.",
		Some(7) => "Tidak bisa terhubung ke server SIPLah. Periksa koneksi internet.",
		Some(22) => "Server SIPLah menolak permintaan (toko tidak ada atau layanan sedang bermasalah).",
		Some(28) => "Server SIPLah tidak menjawab tepat waktu. Coba lagi nanti.",
		Some(35 | 60) => "Gagal membuat koneksi aman (SSL) ke server SIPLah.",
		_ => "Gagal menghubungi server SIPLah.",
	};
	if detail.is_empty() {
		pesan.to_string()
	} else {
		format!("{pesan} ({detail})")
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn curl_errors_are_readable() {
		assert!(curl_error_message(Some(22), "").contains("menolak"));
		assert!(curl_error_message(Some(28), "").contains("tepat waktu"));
		assert_eq!(
			curl_error_message(Some(6), "Could not resolve host"),
			"Server SIPLah tidak ditemukan. Periksa koneksi internet. (Could not resolve host)"
		);
		assert_eq!(curl_error_message(None, ""), "Gagal menghubungi server SIPLah.");
	}

	#[test]
	fn form_values_are_encoded() {
		assert_eq!(encode("cv-futo indonesia&x=@1"), "cv-futo%20indonesia%26x%3D%401");
	}
}
