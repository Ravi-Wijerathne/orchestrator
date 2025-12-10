import { useState } from "react";
import { BrowserRouter as Router, Routes, Route, Link, useLocation } from "react-router-dom";
import {
  LayoutDashboard,
  HardDrive,
  Clock,
  Settings,
  History,
  BarChart3,
} from "lucide-react";

import Dashboard from "./components/Dashboard";
import DriveManager from "./components/DriveManager";
import PendingQueue from "./components/PendingQueue";
import ConfigEditor from "./components/ConfigEditor";
import SyncHistory from "./components/SyncHistory";
import FileTypeStats from "./components/FileTypeStats";

function App() {
  return (
    <Router>
      <div className="flex h-screen bg-gray-900 text-white">
        <Sidebar />
        <main className="flex-1 overflow-auto">
          <Routes>
            <Route path="/" element={<Dashboard />} />
            <Route path="/drives" element={<DriveManager />} />
            <Route path="/pending" element={<PendingQueue />} />
            <Route path="/config" element={<ConfigEditor />} />
            <Route path="/history" element={<SyncHistory />} />
            <Route path="/stats" element={<FileTypeStats />} />
          </Routes>
        </main>
      </div>
    </Router>
  );
}

function Sidebar() {
  const location = useLocation();

  const menuItems = [
    { path: "/", icon: LayoutDashboard, label: "Dashboard" },
    { path: "/drives", icon: HardDrive, label: "Drives" },
    { path: "/pending", icon: Clock, label: "Pending Queue" },
    { path: "/history", icon: History, label: "Sync History" },
    { path: "/stats", icon: BarChart3, label: "Statistics" },
    { path: "/config", icon: Settings, label: "Settings" },
  ];

  return (
    <aside className="w-64 bg-gray-800 border-r border-gray-700 flex flex-col">
      <div className="p-6 border-b border-gray-700">
        <h1 className="text-2xl font-bold text-primary-500">📁 File Orchestrator</h1>
        <p className="text-sm text-gray-400 mt-1">v0.1.0</p>
      </div>

      <nav className="flex-1 p-4 space-y-2">
        {menuItems.map((item) => {
          const Icon = item.icon;
          const isActive = location.pathname === item.path;

          return (
            <Link
              key={item.path}
              to={item.path}
              className={`flex items-center gap-3 px-4 py-3 rounded-lg transition-colors ${
                isActive
                  ? "bg-primary-600 text-white"
                  : "text-gray-300 hover:bg-gray-700 hover:text-white"
              }`}
            >
              <Icon size={20} />
              <span>{item.label}</span>
            </Link>
          );
        })}
      </nav>

      <div className="p-4 border-t border-gray-700">
        <div className="text-xs text-gray-500 text-center">
          Built with Rust + Tauri
        </div>
      </div>
    </aside>
  );
}

export default App;
