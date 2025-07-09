import React from "react";
import { Link } from "react-router-dom";
import ThemeTogglerTwo from "../components/common/ThemeTogglerTwo";

export default function AppHeader() {
  return (
    <header className="sticky top-0 z-50 w-full shadow bg-green-900 dark:bg-green-950">
      <div className="mx-auto max-w-screen-xl px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16">
          {/* Logo */}
          <Link to="/" className="flex items-center space-x-2">
            <img src="/images/logo/simpelv2-kejaksaan.svg" alt="Logo" className="h-8" />
            <span className="text-white text-lg font-bold hidden sm:inline">
              SIMPELv2
            </span>
          </Link>

          {/* Navigasi Utama */}
          <nav className="hidden md:flex items-center space-x-6">
            <Link to="/dashboard" className="text-white hover:text-green-200 font-medium">
              Dashboard
            </Link>
            <Link to="/bank-aset" className="text-white hover:text-green-200 font-medium">
              Bank Aset
            </Link>
            <Link to="/pakaian-dinas" className="text-white hover:text-green-200 font-medium">
              Pakaian Dinas
            </Link>
            <Link to="/roadmap" className="text-white hover:text-green-200 font-medium">
              Roadmap
            </Link>
            <Link to="/laporan" className="text-white hover:text-green-200 font-medium">
              Laporan
            </Link>
          </nav>

          {/* Tema & Pengguna */}
          <div className="flex items-center space-x-4">
            <ThemeTogglerTwo />
            <div className="text-white hidden sm:block">Pengguna</div>
          </div>
        </div>
      </div>
    </header>
  );
}
