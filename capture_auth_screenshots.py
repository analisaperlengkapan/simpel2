import sys
import json
from playwright.sync_api import sync_playwright

def main():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        context = browser.new_context(viewport={'width': 1280, 'height': 800})
        page = context.new_page()

        print("Navigating to index to set origin...")
        page.goto('http://localhost:3000/')

        session_data = {
            "id": "1",
            "username": "admin",
            "role": "Admin",
            "name": "Super Admin",
            "email": "admin@kejaksaan.go.id",
            "avatar": None,
            "division": "Pusat Data Statistik Kriminal dan Teknologi Informasi",
            "captcha_validated": True,
            "mfa_enabled": False,
            "mfa_setup_required": False,
            "created_at": "2024-01-01T00:00:00Z",
            "access_token": "fake_access_token",
            "refresh_token": "fake_refresh_token",
            "expires_at": 9999999999,
            "permissions": ["admin"]
        }

        print("Injecting session into localStorage...")
        page.evaluate(f"localStorage.setItem('user_session', JSON.stringify({json.dumps(session_data)}));")

        pages_to_capture = [
            ('/portal/dashboard', 'dashboard_auth.png'),
            ('/portal/admin', 'admin_panel.png'),
            ('/portal/mfa/setup', 'mfa_setup_auth.png'),
            ('/portal/profile', 'profile_auth.png'),
            ('/portal/settings', 'settings_auth.png'),
            ('/portal/apps', 'apps_auth.png')
        ]

        for url_path, filename in pages_to_capture:
            print(f"Capturing {url_path}...")
            page.goto(f'http://localhost:3000{url_path}')
            page.wait_for_timeout(3000) # Wait for network/hydration
            page.screenshot(path=filename, full_page=True)

        browser.close()
        print("Done capturing authenticated screenshots.")

if __name__ == '__main__':
    main()
