// @ts-check

import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import starlightThemeBlack from "starlight-theme-black";

export default defineConfig({
	site: "https://useskil.dev",
	integrations: [
		starlight({
			title: ">skil",
			social: [
				{
					icon: "github",
					label: "GitHub",
					href: "https://github.com/matoous/skil",
				},
			],
			components: {
				SiteTitle: "./src/overrides/SiteTitle.astro",
			},
			plugins: [
				starlightThemeBlack({
					footerText: "© Matous Dzivjak, 2026",
				}),
			],
			sidebar: [
				{
					label: "Get Started",
					items: [{ label: "Overview", slug: "overview" }],
				},
				{
					label: "Commands",
					autogenerate: { directory: "commands" },
				},
			],
		}),
	],
});
