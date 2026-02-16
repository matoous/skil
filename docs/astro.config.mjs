// @ts-check

import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";

export default defineConfig({
	site: "https://useskil.dev",
	integrations: [
		starlight({
			title: ">skil",
			customCss: ["./src/styles/custom.css"],
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
