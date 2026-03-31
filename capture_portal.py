import asyncio
import os
from playwright.async_api import async_playwright
import json

async def capture_screenshots():
    async with async_playwright() as p:
        browser = await p.chromium.launch()
        context = await browser.new_context(viewport={'width': 1280, 'height': 800})
        page = await context.new_page()

        # Define routes to capture
        routes = [
            {"name": "home", "path": "/portal"},
            {"name": "login", "path": "/portal/login"},
            {"name": "dashboard", "path": "/portal/dashboard"},
            {"name": "apps", "path": "/portal/apps"},
            {"name": "profile", "path": "/portal/profile"},
            {"name": "settings", "path": "/portal/settings"},
            {"name": "notifications", "path": "/portal/notifications"},
            {"name": "admin_overview", "path": "/portal/admin"},
            {"name": "admin_users", "path": "/portal/admin/users"},
            {"name": "admin_roles", "path": "/portal/admin/roles"},
            {"name": "admin_clients", "path": "/portal/admin/clients"},
            {"name": "admin_realms", "path": "/portal/admin/realms"},
            {"name": "admin_federation", "path": "/portal/admin/federation"},
            {"name": "admin_audit", "path": "/portal/admin/audit"},
            {"name": "password", "path": "/portal/password"},
            {"name": "passkeys", "path": "/portal/passkeys"},
            {"name": "sessions", "path": "/portal/sessions"},
        ]

        # Mock session data for an admin user
        mock_session = {
            "id": "admin-id",
            "username": "admin",
            "role": "Admin",
            "name": "Administrator",
            "email": "admin@kejaksaan.go.id",
            "avatar": None,
            "nip": "199203142014031001",
            "jabatan": "System Administrator",
            "satker_code": "0100000",
            "satuan_kerja": "Kejaksaan Agung RI",
            "captcha_validated": True,
            "mfa_enabled": True,
            "mfa_setup_required": False,
            "require_password_change": False,
            "created_at": "2024-01-01T00:00:00Z",
            "access_token": "mock-admin-token",
            "refresh_token": "mock-refresh-token",
            "expires_at": 2524608000, # Year 2050
            "permissions": ["admin:*"]
        }

        # Function to inject session into localStorage
        async def inject_session(page):
            await page.add_init_script(f"""
                localStorage.setItem('user_session', '{json.dumps(mock_session)}');
                localStorage.setItem('auth_token', 'mock-admin-token');
            """)

        for route in routes:
            url = f"http://localhost:8080{route['path']}"
            print(f"Capturing {route['name']} at {url}...")

            # Re-inject session for each page load to ensure it stays
            await inject_session(page)

            try:
                await page.goto(url, wait_until="networkidle")
                # Wait a bit more for Leptos to render
                await asyncio.sleep(2)
                await page.screenshot(path=f"{route['name']}.png", full_page=True)
            except Exception as e:
                print(f"Failed to capture {route['name']}: {e}")

        await browser.close()

if __name__ == "__main__":
    asyncio.run(capture_screenshots())
