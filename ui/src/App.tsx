import { useState, useEffect } from "react";
import HyperwareClientApi from "@hyperware-ai/client-api";
import "./App.css";
import { WikiList } from "./components/WikiList";
import { WikiPage } from "./components/WikiPage";
import { InviteCenter } from "./components/InviteCenter";
import { useWikiStore } from "./store/wikiStore";
import { useTheme } from "./context/ThemeContext";

const BASE_URL = import.meta.env.BASE_URL;
if (window.our) window.our.process = BASE_URL?.replace("/", "");

const PROXY_TARGET = `${(import.meta.env.VITE_NODE_URL || "http://localhost:8080")}${BASE_URL}`;

// This env also has BASE_URL which should match the process + package name
const WEBSOCKET_URL = import.meta.env.DEV
  ? `${PROXY_TARGET.replace('http', 'ws')}`
  : undefined;

function App() {
  const { currentWiki } = useWikiStore();
  const { isDarkMode, toggleTheme } = useTheme();
  const [nodeConnected, setNodeConnected] = useState(true);
  const [api, setApi] = useState<HyperwareClientApi | undefined>();
  const [showInviteCenter, setShowInviteCenter] = useState(false);

  useEffect(() => {
    // Connect to the Hyperdrive via websocket
    console.log('WEBSOCKET URL', WEBSOCKET_URL)
    if (window.our?.node && window.our?.process) {
      const api = new HyperwareClientApi({
        uri: WEBSOCKET_URL,
        nodeId: window.our.node,
        processId: window.our.process,
        onOpen: (_event, _api) => {
          console.log("Connected to Hyperware");
        },
        onMessage: (json, _api) => {
          console.log('WEBSOCKET MESSAGE', json)
          try {
            const data = JSON.parse(json);
            console.log("WebSocket received message", data);
          } catch (error) {
            console.error("Error parsing WebSocket message", error);
          }
        },
      });

      setApi(api);
    } else {
      setNodeConnected(false);
    }
  }, []);

  return (
    <div className="app">
      <header className="app-header">
        <h1>Hyperware Wiki</h1>
        <div style={{ display: 'flex', alignItems: 'center', gap: '1rem' }}>
          <button 
            onClick={toggleTheme}
            style={{ 
              padding: '0.5rem',
              fontSize: '1.2rem',
              background: 'transparent',
              border: '1px solid var(--border-color)',
              borderRadius: '4px',
              cursor: 'pointer'
            }}
            title={isDarkMode ? 'Switch to light mode' : 'Switch to dark mode'}
          >
            {isDarkMode ? '☀️' : '🌙'}
          </button>
          <div 
            className="node-info clickable"
            onClick={() => setShowInviteCenter(!showInviteCenter)}
            title="Click to view invites"
          >
            Node: <strong>{window.our?.node || "Not connected"}</strong>
          </div>
        </div>
      </header>
      
      {!nodeConnected && (
        <div className="node-not-connected">
          <h2 style={{ color: "red" }}>Node not connected</h2>
          <h4>
            You need to start a node at {PROXY_TARGET} before you can use this UI
            in development.
          </h4>
        </div>
      )}
      
      <main className="app-main">
        {currentWiki ? <WikiPage /> : <WikiList />}
      </main>

      {showInviteCenter && (
        <div className="invite-center-modal" onClick={() => setShowInviteCenter(false)}>
          <div className="invite-center-modal-content" onClick={(e) => e.stopPropagation()}>
            <button 
              className="close-btn"
              onClick={() => setShowInviteCenter(false)}
              aria-label="Close"
            >
              ×
            </button>
            <InviteCenter />
          </div>
        </div>
      )}
    </div>
  );
}

export default App;