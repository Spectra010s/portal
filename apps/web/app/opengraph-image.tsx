import { ImageResponse } from "next/og";

export const runtime = "edge";
export const alt = "Portal - Local-first file transfer";
export const size = {
  width: 1200,
  height: 630,
};
export const contentType = "image/png";

export default function Image() {
  const site = (
    process.env.NEXT_PUBLIC_SITE_URL || "https://portal.build.app"
  ).replace(/^https?:\/\//, "");

  return new ImageResponse(
    (
      <div
        style={{
          width: "100%",
          height: "100%",
          display: "flex",
          flexDirection: "column",
          justifyContent: "space-between",
          padding: "56px 64px",
          background:
            "linear-gradient(135deg, #020617 0%, #0f172a 45%, #0c4a6e 100%)",
          color: "#e2e8f0",
          fontFamily: "Inter, ui-sans-serif, system-ui",
          position: "relative",
          overflow: "hidden",
        }}
      >
        <div
          style={{
            position: "absolute",
            top: -140,
            right: -80,
            width: 460,
            height: 460,
            borderRadius: "9999px",
            background:
              "radial-gradient(circle, rgba(56,189,248,0.25) 0%, rgba(56,189,248,0) 70%)",
            display: "flex",
          }}
        />

        <div style={{ display: "flex", flexDirection: "column", gap: 18 }}>
          <div
            style={{
              display: "flex",
              alignItems: "center",
              gap: 12,
              fontSize: 24,
              letterSpacing: "0.18em",
              textTransform: "uppercase",
              color: "#7dd3fc",
              fontWeight: 700,
            }}
          >
            <span
              style={{
                width: 12,
                height: 12,
                borderRadius: 9999,
                background: "#38bdf8",
                display: "flex",
              }}
            />
            Hiverra Portal
          </div>

          <div
            style={{
              display: "flex",
              flexDirection: "column",
              fontSize: 90,
              lineHeight: 0.94,
              letterSpacing: "-0.05em",
              fontWeight: 800,
              color: "#f8fafc",
            }}
          >
            <span>Move files across</span>
            <span>devices directly.</span>
          </div>

          <p
            style={{
              margin: 0,
              marginTop: 8,
              maxWidth: 840,
              fontSize: 32,
              lineHeight: 1.35,
              color: "#cbd5e1",
            }}
          >
            Local-first CLI transfers with discovery, direct IP mode, and
            transfer history.
          </p>
        </div>

        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            borderTop: "1px solid rgba(148,163,184,0.28)",
            paddingTop: 26,
            fontSize: 24,
            color: "#94a3b8",
          }}
        >
          <span style={{ display: "flex" }}>{site}</span>
          <span
            style={{
              display: "flex",
              padding: "8px 16px",
              borderRadius: 9999,
              border: "1px solid rgba(125,211,252,0.45)",
              color: "#bae6fd",
              fontSize: 20,
              letterSpacing: "0.08em",
              textTransform: "uppercase",
              fontWeight: 700,
            }}
          >
            CLI-first
          </span>
        </div>
      </div>
    ),
    size,
  );
}
