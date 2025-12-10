import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { PieChart, Pie, Cell, ResponsiveContainer, Legend, Tooltip, BarChart, Bar, XAxis, YAxis, CartesianGrid } from "recharts";
import { BarChart3, RefreshCw } from "lucide-react";

interface FileTypeStats {
  category: string;
  count: number;
  total_size: number;
  percentage: number;
}

const COLORS = {
  documents: "#2196f3",
  images: "#4caf50",
  videos: "#9c27b0",
  audio: "#ff9800",
  archives: "#f44336",
};

export default function FileTypeStats() {
  const [stats, setStats] = useState<FileTypeStats[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadStats();
    const interval = setInterval(loadStats, 10000);
    return () => clearInterval(interval);
  }, []);

  const loadStats = async () => {
    try {
      const data = await invoke<FileTypeStats[]>("get_file_type_stats");
      setStats(data);
    } catch (error) {
      console.error("Failed to load stats:", error);
    } finally {
      setLoading(false);
    }
  };

  const pieData = stats.map(stat => ({
    name: stat.category.charAt(0).toUpperCase() + stat.category.slice(1),
    value: stat.count,
    percentage: stat.percentage,
  }));

  const barData = stats.map(stat => ({
    name: stat.category.charAt(0).toUpperCase() + stat.category.slice(1),
    count: stat.count,
  }));

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500"></div>
      </div>
    );
  }

  return (
    <div className="p-8">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold mb-2">File Type Statistics</h1>
          <p className="text-gray-400">
            Analyze your synced files by category and type
          </p>
        </div>
        <button
          onClick={loadStats}
          className="flex items-center gap-2 px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded-lg transition-colors"
        >
          <RefreshCw size={20} />
          Refresh
        </button>
      </div>

      {stats.length === 0 ? (
        <div className="text-center py-16">
          <BarChart3 size={64} className="mx-auto mb-4 text-gray-600" />
          <h3 className="text-xl font-semibold mb-2">No data available</h3>
          <p className="text-gray-400">
            Statistics will appear here after files are synced
          </p>
        </div>
      ) : (
        <>
          {/* Summary Cards */}
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-5 gap-4 mb-8">
            {stats.map((stat) => (
              <div
                key={stat.category}
                className="bg-gray-800 rounded-lg p-4 border border-gray-700"
              >
                <div
                  className="w-3 h-3 rounded-full mb-2"
                  style={{ backgroundColor: COLORS[stat.category as keyof typeof COLORS] || "#888" }}
                ></div>
                <div className="text-2xl font-bold mb-1">{stat.count}</div>
                <div className="text-sm text-gray-400 capitalize">
                  {stat.category}
                </div>
                <div className="text-xs text-gray-500 mt-1">
                  {stat.percentage.toFixed(1)}%
                </div>
              </div>
            ))}
          </div>

          {/* Charts */}
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            {/* Pie Chart */}
            <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
              <h2 className="text-xl font-semibold mb-4">Distribution by Category</h2>
              <ResponsiveContainer width="100%" height={300}>
                <PieChart>
                  <Pie
                    data={pieData}
                    cx="50%"
                    cy="50%"
                    labelLine={false}
                    label={({ name, percentage }) => `${name}: ${percentage.toFixed(1)}%`}
                    outerRadius={100}
                    fill="#8884d8"
                    dataKey="value"
                  >
                    {pieData.map((entry, index) => {
                      const category = stats[index].category;
                      return (
                        <Cell
                          key={`cell-${index}`}
                          fill={COLORS[category as keyof typeof COLORS] || "#888"}
                        />
                      );
                    })}
                  </Pie>
                  <Tooltip
                    contentStyle={{
                      backgroundColor: "#1f2937",
                      border: "1px solid #374151",
                      borderRadius: "8px",
                    }}
                  />
                  <Legend />
                </PieChart>
              </ResponsiveContainer>
            </div>

            {/* Bar Chart */}
            <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
              <h2 className="text-xl font-semibold mb-4">File Count by Category</h2>
              <ResponsiveContainer width="100%" height={300}>
                <BarChart data={barData}>
                  <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
                  <XAxis dataKey="name" stroke="#9ca3af" />
                  <YAxis stroke="#9ca3af" />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: "#1f2937",
                      border: "1px solid #374151",
                      borderRadius: "8px",
                    }}
                  />
                  <Bar dataKey="count" fill="#2196f3" radius={[8, 8, 0, 0]} />
                </BarChart>
              </ResponsiveContainer>
            </div>
          </div>

          {/* Detailed Table */}
          <div className="mt-6 bg-gray-800 rounded-lg border border-gray-700 overflow-hidden">
            <table className="w-full">
              <thead className="bg-gray-700">
                <tr>
                  <th className="px-6 py-3 text-left text-sm font-semibold">Category</th>
                  <th className="px-6 py-3 text-right text-sm font-semibold">Count</th>
                  <th className="px-6 py-3 text-right text-sm font-semibold">Percentage</th>
                  <th className="px-6 py-3 text-left text-sm font-semibold">Visual</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-700">
                {stats.map((stat) => (
                  <tr key={stat.category} className="hover:bg-gray-700/50">
                    <td className="px-6 py-4">
                      <div className="flex items-center gap-3">
                        <div
                          className="w-3 h-3 rounded-full"
                          style={{ backgroundColor: COLORS[stat.category as keyof typeof COLORS] || "#888" }}
                        ></div>
                        <span className="capitalize font-medium">{stat.category}</span>
                      </div>
                    </td>
                    <td className="px-6 py-4 text-right font-semibold">
                      {stat.count.toLocaleString()}
                    </td>
                    <td className="px-6 py-4 text-right text-gray-400">
                      {stat.percentage.toFixed(2)}%
                    </td>
                    <td className="px-6 py-4">
                      <div className="w-full bg-gray-700 rounded-full h-2">
                        <div
                          className="h-2 rounded-full transition-all"
                          style={{
                            width: `${stat.percentage}%`,
                            backgroundColor: COLORS[stat.category as keyof typeof COLORS] || "#888",
                          }}
                        ></div>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </>
      )}
    </div>
  );
}
