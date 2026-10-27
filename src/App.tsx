import { useState } from "react";
import { TabRail, type TabId } from "@/components/TabRail";
import { ConverterTab } from "@/components/tabs/ConverterTab";
import { FilesTab } from "@/components/tabs/FilesTab";
import { FutureTab } from "@/components/tabs/FutureTab";

export default function App() {
  const [activeTab, setActiveTab] = useState<TabId>("converter");

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-surface">
      <TabRail active={activeTab} onChange={setActiveTab} />
      <main className="flex-1 overflow-hidden px-8 py-6">
        {activeTab === "converter" && <ConverterTab />}
        {activeTab === "files" && <FilesTab />}
        {activeTab === "future" && <FutureTab />}
      </main>
    </div>
  );
}
