/** WCAG constrast ratio calculator */

import type { SystemModeValue } from "mode-watcher";

const TARGET_CONTRAST_RATIO = 5.5;

function getRelativeLum(r: number, g: number, b: number) {
	const [rs, gs, bs] = [r, g, b].map((c) => {
		c = c / 255;
		return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
	});

	return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
}

function getContrastRatio(lum1: number, lum2: number) {
	const lighter = Math.max(lum1, lum2);
	const darker = Math.min(lum1, lum2);

	return (lighter + 0.05) / (darker + 0.05);
}

function cmp(
	r: number,
	g: number,
	b: number,
	bg = "black",
	minContrast = TARGET_CONTRAST_RATIO
) {
	const bgLum = bg === "black" ? 0 : 1;

	let lum = getRelativeLum(r, g, b);
	let ratio = getContrastRatio(lum, bgLum);

	if (ratio >= minContrast) {
		return { r, g, b };
	}

	let [h, s, l] = rgbToHsl(r, g, b);

	let lo;
	let hi;
	if (bg === "black") {
		lo = l;
		hi = 1;
	} else {
		lo = 0;
		hi = l;
	}

	let result =
		bg === "black" ? { r: 255, g: 255, b: 255 } : { r: 0, g: 0, b: 0 };

	for (let i = 0; i < 50; i++) {
		const mid: number = (lo + hi) / 2;

		const [nr, ng, nb] = hslToRgb(h, s, mid);
		const midLum = getRelativeLum(nr, ng, nb);
		const midRatio = getContrastRatio(midLum, bgLum);

		if (midRatio >= minContrast) {
			result = { r: nr, g: ng, b: nb };
			if (bg === "black") {
				hi = mid;
			} else {
				lo = mid;
			}
		} else {
			if (bg === "black") {
				lo = mid;
			} else {
				hi = mid;
			}
		}
	}

	return result;
}

function rgbToHsl(r: number, g: number, b: number) {
	r /= 255;
	g /= 255;
	b /= 255;

	const max = Math.max(r, g, b);
	const min = Math.min(r, g, b);

	let h = (max + min) / 2;
	let s = (max + min) / 2;
	let l = (max + min) / 2;

	if (max === min) {
		h = s = 0;
	} else {
		const d = max - min;
		s = l > 0.5 ? d / (2 - max - min) : d / (max + min);
		switch (max) {
			case r:
				h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
				break;
			case g:
				h = ((b - r) / d + 2) / 6;
				break;
			case b:
				h = ((r - g) / d + 4) / 6;
				break;
		}
	}
	return [h, s, l];
}

function hslToRgb(h: number, s: number, l: number) {
	let r;
	let g;
	let b;
	if (s === 0) {
		r = l;
		g = l;
		b = l;
	} else {
		const hue2rgb = (p: number, q: number, t: number) => {
			if (t < 0) t += 1;
			if (t > 1) t -= 1;
			if (t < 1 / 6) return p + (q - p) * 6 * t;
			if (t < 1 / 2) return q;
			if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
			return p;
		};
		const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
		const p = 2 * l - q;
		r = hue2rgb(p, q, h + 1 / 3);
		g = hue2rgb(p, q, h);
		b = hue2rgb(p, q, h - 1 / 3);
	}
	return [Math.round(r * 255), Math.round(g * 255), Math.round(b * 255)];
}

function hexToRgb(hex: string) {
	const shorthandRegex = /^#?([a-f\d])([a-f\d])([a-f\d])$/i;
	hex = hex.replace(shorthandRegex, (_, r, g, b) => {
		return r + r + g + g + b + b;
	});

	const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
	return result
		? {
				r: parseInt(result[1], 16),
				g: parseInt(result[2], 16),
				b: parseInt(result[3], 16)
			}
		: null;
}

function rgbToHex(r: number, g: number, b: number) {
	return (
		"#" +
		[r, g, b]
			.map((p) => {
				const hex = p.toString(16);
				return hex.length === 1 ? "0" + hex : hex;
			})
			.join("")
			.toUpperCase()
	);
}

export function readableColor(color: string, mode: SystemModeValue = "dark", minContrast?: number) {
	const bg = mode === "dark" ? "black" : "white";
	const { r, g, b } = hexToRgb(color)!;
	const rgbOut = cmp(r, g, b, bg, minContrast);

	return rgbToHex(rgbOut.r, rgbOut.g, rgbOut.b);
}
