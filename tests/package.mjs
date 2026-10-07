import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

import init, { ComparisonMethod, convert, convert_history } from "../pkg/converter.js";

const wasm = await readFile(new URL("../pkg/converter_bg.wasm", import.meta.url));
await init({ module_or_path: wasm });

test("the generated JavaScript package converts a LiveSplit file", async () => {
	const input = await readFile(
		new URL("./fixtures/sa2_fallen-hero.lss", import.meta.url),
		"utf8",
	);
	const output = convert(input, ComparisonMethod.GameTime);

	assert.equal(typeof output, "string");
	assert.ok(output.length > 0);
});

test("the generated JavaScript package converts history into a ZIP Blob", async () => {
	const input = '<Run><AttemptHistory><Attempt id="1"><RealTime>00:00:03</RealTime></Attempt></AttemptHistory><Segments /></Run>';
	const output = convert_history(input, 'dir_name');

	assert.ok(output instanceof Blob);
	assert.equal(output.type, "application/zip");
	assert.ok((await output.arrayBuffer()).byteLength > 0);
	assert.throws(() => convert_history(input, "../splits"));
});