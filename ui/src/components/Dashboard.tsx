import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import {
  Play,
  Pause,
  RefreshCw,
  HardDrive,
  FileText,
  Clock,
  Activity,
  FolderOpen,
} from "lucide-react";

interface DashboardStats {
  total_syncs: number;
  pending_files: number;
  registered_drives: number;
  connected_drives: number;
  total_file_types: number;
  is_watching: boolean;
  source_directory: string;
}

export default function Dashboard() {
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [syncing, setSyncing] = useState(false);

  useEffect(() => {
    loadStats();
    const interval = setInterval(loadStats, 5000);

    // Listen for file sync events
    const unlisten = listen("file-synced", () => {
      loadStats();
    });

    return () => {
      clearInterval(interval);
      unlisten.then((fn) => fn());
    };
  }, []);

  const loadStats = async () => {
    try {
      const data = await invoke<DashboardStats>("get_dashboard_stats");
      setStats(data);
    } catch (error) {
      console.error("Failed to load stats:", error);
    } finally {
      setLoading(false);
    }
  };

  const toggleWatching = async () => {
    try {
      if (stats?.is_watching) {
        await invoke("stop_watching");
      } else {
        await invoke("start_watching");
      }
      await loadStats();
    } catch (error) {
      console.error("Failed to toggle watching:", error);
    }
  };

  const syncPending = async () => {
    setSyncing(true);
    try {
      const count = await invoke<number>("sync_pending_cmd");
      alert(`Successfully synced ${count} pending files`);
      await loadStats();
    } catch (error) {
      console.error("Failed to sync pending:", error);
      alert("Failed to sync pending files");
    } finally {
      setSyncing(false);
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500"></div>
      </div>
    );
  }

  return (
    <div className="p-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold mb-2">Dashboard</h1>
        <p className="text-gray-400">Real-time overview of your file orchestration</p>
      </div>

      {/* Control Panel */}
      <div className="bg-gray-800 rounded-lg p-6 mb-6 border border-gray-700">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-xl font-semibold mb-1">File Watching</h2>
            <p className="text-sm text-gray-400">
              <FolderOpen className="inline mr-2" size={16} />
              {stats?.source_directory || "Not configured"}
            </p>
          </div>
          <div className="flex gap-3">
            <button
              onClick={toggleWatching}
              className={`flex items-center gap-2 px-6 py-3 rounded-lg font-semibold transition-colors ${
                stats?.is_watching
                  ? "bg-red-600 hover:bg-red-700"
                  : "bg-green-600 hover:bg-green-700"
              }`}
            >
              {stats?.is_watching ? (
                <>
                  <Pause size={20} />
                  Stop Watching
                </>
              ) : (
                <>
                  <Play size={20} />
                  Start Watching
                </>
              )}
            </button>
            <button
              onClick={syncPending}
              disabled={syncing || stats?.pending_files === 0}
              className="flex items-center gap-2 px-6 py-3 rounded-lg font-semibold bg-primary-600 hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              <RefreshCw size={20} className={syncing ? "animate-spin" : ""} />
              Sync Pending
            </button>
          </div>
        </div>

        {stats?.is_watching && (
          <div className="mt-4 flex items-center gap-2 text-green-400">
            <Activity size={16} className="animate-pulse" />
            <span className="text-sm">Actively monitoring for new files</span>
          </div>
        )}
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <StatCard
          icon={<FileText className="text-primary-500" />}
          label="Total Syncs"
          value={stats?.total_syncs || 0}
          color="primary"
        />
        <StatCard
          icon={<Clock className="text-yellow-500" />}
          label="Pending Files"
          value={stats?.pending_files || 0}
          color="yellow"
        />
        <StatCard
          icon={<HardDrive className="text-green-500" />}
          label="Connected Drives"
          value={`${stats?.connected_drives || 0} / ${stats?.registered_drives || 0}`}
          color="green"
        />
        <StatCard
          icon={<Activity className="text-purple-500" />}
          label="File Types"
          value={stats?.total_file_types || 0}
          color="purple"
        />
      </div>

      {/* Recent Activity */}
      <div className="mt-8 bg-gray-800 rounded-lg p-6 border border-gray-700">
        <h2 className="text-xl font-semibold mb-4">System Status</h2>
        <div className="space-y-3">
          <StatusRow
            label="File Watcher"
            status={stats?.is_watching ? "Running" : "Stopped"}
            isActive={stats?.is_watching || false}
          />
          <StatusRow
            label="Registered Drives"
            status={`${stats?.registered_drives || 0} configured`}
            isActive={(stats?.registered_drives || 0) > 0}
          />
          <StatusRow
            label="Connected Drives"
            status={`${stats?.connected_drives || 0} online`}
            isActive={(stats?.connected_drives || 0) > 0}
          />
        </div>
      </div>
    </div>
  );
}

interface StatCardProps {
  icon: React.ReactNode;
  label: string;
  value: string | number;
  color: string;
}

function StatCard({ icon, label, value, color }: StatCardProps) {
  return (
    <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
      <div className="flex items-start justify-between mb-4">
        <div className="p-3 bg-gray-700 rounded-lg">{icon}</div>
      </div>
      <div className="text-3xl font-bold mb-1">{value}</div>
      <div className="text-sm text-gray-400">{label}</div>
    </div>
  );
}

interface StatusRowProps {
  label: string;
  status: string;
  isActive: boolean;
}

function StatusRow({ label, status, isActive }: StatusRowProps) {
  return (
    <div className="flex items-center justify-between py-2">
      <span className="text-gray-300">{label}</span>
      <div className="flex items-center gap-2">
        <span
          className={`w-2 h-2 rounded-full ${
            isActive ? "bg-green-500" : "bg-gray-500"
          }`}
        ></span>
        <span className={isActive ? "text-green-400" : "text-gray-400"}>
          {status}
        </span>
      </div>
    </div>
  );
}
