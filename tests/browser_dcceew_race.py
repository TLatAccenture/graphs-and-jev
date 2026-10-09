#!/usr/bin/env python3
import asyncio
import os

from playwright.async_api import async_playwright


async def main() -> None:
    base_url = os.environ.get("BASE_URL", "http://127.0.0.1:43128")
    pipeline_requests = []

    async with async_playwright() as playwright:
        browser = await playwright.chromium.launch(
            executable_path="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            headless=True,
        )
        page = await browser.new_page()

        async def route_api(route):
            if route.request.url.endswith("/api/catalogue"):
                await route.fulfill(json={"summary": "Test catalogue.", "sources": []})
                return
            body = route.request.post_data_json
            pipeline_requests.append(body["request"])
            await asyncio.sleep(0.2)
            await route.fulfill(json={
                "outcome": "clarify",
                "message": "First request completed.",
                "stages": [],
            })

        await page.route("**/api/catalogue", route_api)
        await page.route("**/api/pipeline", route_api)
        await page.goto(f"{base_url}/dcceew/")
        scenarios = page.locator("[data-q]")
        await scenarios.nth(0).click()
        await page.wait_for_function("document.querySelector('#ask').getAttribute('aria-busy') === 'true'")
        await scenarios.nth(1).evaluate("button => button.click()")
        await page.wait_for_function("!document.querySelector('#ask').hasAttribute('aria-busy')")

        assert pipeline_requests == ["Annual PM2.5 by country, 2019 to 2024"], pipeline_requests
        assert await scenarios.nth(0).get_attribute("aria-pressed") == "true"
        assert await scenarios.nth(1).get_attribute("aria-pressed") == "false"
        assert not await scenarios.nth(0).is_disabled()
        assert not await scenarios.nth(1).is_disabled()
        assert await page.locator("#status").get_attribute("data-state") == "done"
        await browser.close()


if __name__ == "__main__":
    asyncio.run(main())
