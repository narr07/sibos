//! Helper kecil untuk membaca HTML halaman toko SIPLah tanpa dependensi parser tambahan.
//! Halaman yang dibaca bentuknya tetap (template server), jadi cukup pencarian teks berpatokan.

/// Teks di antara `start` dan `end` (yang pertama setelah `start`).
pub fn between<'a>(hay: &'a str, start: &str, end: &str) -> Option<&'a str> {
	let s = hay.find(start)? + start.len();
	let e = hay[s..].find(end)? + s;
	Some(&hay[s..e])
}

/// Nilai atribut pertama `attr="..."` setelah posisi awal teks.
pub fn attr<'a>(tag: &'a str, attr: &str) -> Option<&'a str> {
	between(tag, &format!("{attr}=\""), "\"")
}

/// Buang tag HTML, ubah entitas, dan rapikan spasi.
pub fn text(html: &str) -> String {
	let mut out = String::with_capacity(html.len());
	let mut in_tag = false;
	for c in html.chars() {
		match c {
			'<' => in_tag = true,
			'>' if in_tag => {
				in_tag = false;
				out.push(' ');
			}
			_ if !in_tag => out.push(c),
			_ => {}
		}
	}
	decode_entities(&out).split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Entitas HTML yang umum muncul di nama produk dan alamat.
pub fn decode_entities(s: &str) -> String {
	if !s.contains('&') {
		return s.to_string();
	}
	let mut out = String::with_capacity(s.len());
	let mut rest = s;
	while let Some(pos) = rest.find('&') {
		out.push_str(&rest[..pos]);
		rest = &rest[pos..];
		let Some(end) = rest.find(';').filter(|e| *e <= 10) else {
			out.push('&');
			rest = &rest[1..];
			continue;
		};
		let ent = &rest[1..end];
		let decoded = match ent {
			"amp" => Some('&'),
			"lt" => Some('<'),
			"gt" => Some('>'),
			"quot" => Some('"'),
			"apos" | "#39" => Some('\''),
			"nbsp" => Some(' '),
			_ if ent.starts_with("#x") || ent.starts_with("#X") => u32::from_str_radix(&ent[2..], 16).ok().and_then(char::from_u32),
			_ if ent.starts_with('#') => ent[1..].parse::<u32>().ok().and_then(char::from_u32),
			_ => None,
		};
		match decoded {
			Some(c) => {
				out.push(c);
				rest = &rest[end + 1..];
			}
			None => {
				out.push('&');
				rest = &rest[1..];
			}
		}
	}
	out.push_str(rest);
	out
}

/// "Rp. 86.900" / "Rp130.000" / "Rp 1.250,50" -> 86900 / 130000 / 1250 (pecahan sen dibuang).
pub fn rupiah(s: &str) -> Option<i64> {
	let s = s.trim();
	let start = s.find(|c: char| c.is_ascii_digit())?;
	let digits: String = s[start..]
		.chars()
		.take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
		.collect();
	let integer = digits.split(',').next().unwrap_or("");
	integer.replace('.', "").parse().ok()
}

/// "Rp. 86.900 - Rp. 181.100" -> (86900, Some(181100)); "Rp130.000" -> (130000, None).
pub fn rentang_rupiah(s: &str) -> Option<(i64, Option<i64>)> {
	let mut parts = s.splitn(2, " - ");
	let min = rupiah(parts.next()?)?;
	let max = parts.next().and_then(rupiah).filter(|max| *max > min);
	Some((min, max))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn reads_rupiah_formats() {
		assert_eq!(rupiah("Rp. 86.900"), Some(86_900));
		assert_eq!(rupiah("Rp130.000"), Some(130_000));
		assert_eq!(rupiah("Rp 1.250,50"), Some(1_250));
		assert_eq!(rupiah("Gratis"), None);
		assert_eq!(rentang_rupiah("Rp. 86.900 - Rp. 181.100"), Some((86_900, Some(181_100))));
		assert_eq!(rentang_rupiah("Rp. 5.000 - Rp. 5.000"), Some((5_000, None)));
		assert_eq!(rentang_rupiah("Rp130.000"), Some((130_000, None)));
	}

	#[test]
	fn text_strips_tags_and_entities() {
		assert_eq!(text("<b>Pensil&nbsp;2B</b>\n  &amp; <i>Penghapus</i> &#39;A&#x27;"), "Pensil 2B & Penghapus 'A'");
		assert_eq!(decode_entities("A & B &unknown; C"), "A & B &unknown; C");
	}

	#[test]
	fn between_and_attr() {
		assert_eq!(between("<h4>Toko</h4>", "<h4>", "</h4>"), Some("Toko"));
		assert_eq!(attr("<img loading=\"lazy\" src=\"x.png\">", "src"), Some("x.png"));
		assert_eq!(between("abc", "x", "c"), None);
	}
}
