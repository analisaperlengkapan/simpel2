<?php

namespace App\Services;

use App\Models\Pengguna\Aktifitas;

/**
 * Session Management Service
 *
 * Handles cross-tab logout via localStorage broadcast events
 * Uses canonical localStorage keys: auth_token, logout_event
 */
class SessionService
{
    /**
     * Broadcast logout event across tabs
     *
     * Called when user explicitly logs out in one tab
     */
    public static function broadcastLogout(): void
    {
        // Set logout event marker in session for detection
        session()->put('logout_broadcast_at', now()->toIso8601String());

        // Client-side will listen to storage event and sync logout
        // No server-side cross-tab communication needed (localStorage handles it)
    }

    /**
     * Handle logout via storage event (from another tab)
     */
    public static function handleRemoteLogout(): void
    {
        if (auth()->check()) {
            // Log activity
            Aktifitas::create([
                'username' => auth()->user()->username,
                'operation' => 'LOGOUT_REMOTE',
                'table' => 'sessions',
                'keterangan' => 'Session invalidated from another tab',
                'ip_address' => request()->ip(),
            ]);

            // Invalidate session
            auth()->logout();
            session()->flush();
        }
    }

    /**
     * Get current session token
     */
    public static function getSessionToken(): ?string
    {
        return session()->get('auth_token');
    }

    /**
     * Verify session is still valid
     */
    public static function isSessionValid(): bool
    {
        return auth()->check() && session()->has('auth_token');
    }
}
