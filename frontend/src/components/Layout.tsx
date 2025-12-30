/**
 * Main layout component with pump.fun style navigation
 */

import { useState } from 'react';
import { Link, NavLink, Outlet } from 'react-router-dom';
import { Rocket, Menu, X } from 'lucide-react';
import { WalletButton } from './WalletButton';

export function Layout() {
  const [mobileMenuOpen, setMobileMenuOpen] = useState(false);

  return (
    <div className="min-h-screen bg-[#0F1014]">
      {/* Header */}
      <header className="sticky top-0 z-50 border-b border-white/5 bg-[#0F1014]/95 backdrop-blur-xl">
        <div className="container mx-auto px-4">
          <div className="flex items-center justify-between h-16">
            {/* Logo */}
            <Link to="/" className="flex items-center gap-2 group">
              <Rocket className="w-6 h-6 text-purple-500 group-hover:scale-110 transition-transform" />
              <span className="text-xl font-black bg-gradient-to-r from-white via-purple-200 to-purple-400 bg-clip-text text-transparent">
                Linera.fun
              </span>
            </Link>

            {/* Desktop Navigation */}
            <nav className="hidden md:flex items-center gap-6">
              <NavItem to="/" label="Board" />
              <NavItem to="/dex" label="DEX" />
              <NavItem to="/how-it-works" label="How it works" />
            </nav>

            {/* Desktop Actions */}
            <div className="hidden md:flex items-center gap-3">
              <Link
                to="/create"
                className="px-4 py-2 rounded-lg bg-purple-600 hover:bg-purple-700 text-white text-sm font-bold transition-all hover:scale-105"
              >
                Start a new coin
              </Link>
              <WalletButton />
            </div>

            {/* Mobile Menu Toggle */}
            <button
              className="md:hidden p-2 text-gray-400 hover:text-white"
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
            >
              {mobileMenuOpen ? <X className="w-6 h-6" /> : <Menu className="w-6 h-6" />}
            </button>
          </div>
        </div>

        {/* Mobile Menu */}
        {mobileMenuOpen && (
          <div className="md:hidden border-t border-white/5 bg-[#16171D]">
            <nav className="container mx-auto px-4 py-4 flex flex-col gap-2">
              <MobileNavItem to="/" label="Board" onClick={() => setMobileMenuOpen(false)} />
              <MobileNavItem to="/dex" label="DEX" onClick={() => setMobileMenuOpen(false)} />
              <MobileNavItem to="/how-it-works" label="How it works" onClick={() => setMobileMenuOpen(false)} />
              <Link
                to="/create"
                className="px-4 py-2 rounded-lg bg-purple-600 text-white text-sm font-bold text-center"
                onClick={() => setMobileMenuOpen(false)}
              >
                Start a new coin
              </Link>
              <div className="pt-2">
                <WalletButton />
              </div>
            </nav>
          </div>
        )}
      </header>

      {/* Main Content */}
      <main className="container mx-auto px-4 py-8">
        <Outlet />
      </main>

      {/* Footer */}
      <footer className="border-t border-white/5 mt-16 bg-[#16171D]">
        <div className="container mx-auto px-4 py-6">
          <div className="flex flex-col md:flex-row items-center justify-between gap-4">
            <div className="text-sm text-gray-500 flex items-center gap-2">
              Built on Linera Microchains ⚡ Real-time Architecture
            </div>
            <div className="flex items-center gap-6">
              <a
                href="https://linera.dev"
                target="_blank"
                rel="noopener noreferrer"
                className="text-sm text-gray-500 hover:text-purple-400 transition-colors"
              >
                Docs
              </a>
              <a
                href="https://github.com"
                target="_blank"
                rel="noopener noreferrer"
                className="text-sm text-gray-500 hover:text-purple-400 transition-colors"
              >
                GitHub
              </a>
              <a
                href="https://discord.gg/linera"
                target="_blank"
                rel="noopener noreferrer"
                className="text-sm text-gray-500 hover:text-purple-400 transition-colors"
              >
                Discord
              </a>
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
}

function NavItem({ to, label }: { to: string; label: string }) {
  return (
    <NavLink
      to={to}
      className={({ isActive }) =>
        `text-sm font-medium transition-colors ${
          isActive
            ? 'text-white'
            : 'text-gray-400 hover:text-white'
        }`
      }
    >
      {label}
    </NavLink>
  );
}

function MobileNavItem({
  to,
  label,
  onClick,
}: {
  to: string;
  label: string;
  onClick: () => void;
}) {
  return (
    <NavLink
      to={to}
      onClick={onClick}
      className={({ isActive }) =>
        `px-4 py-2 rounded-lg text-sm font-medium transition-all ${
          isActive
            ? 'bg-white/10 text-white'
            : 'text-gray-400 hover:bg-white/5 hover:text-white'
        }`
      }
    >
      {label}
    </NavLink>
  );
}
