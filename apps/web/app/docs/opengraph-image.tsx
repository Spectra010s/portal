import { ImageResponse } from "next/og";

export const runtime = "edge";
export const alt = "Portal Docs - CLI transfer workflows";
export const size = {
  width: 1200,
  height: 630,
};
export const contentType = "image/png";

export default function Image() {
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
            "linear-gradient(145deg, #f8fafc 0%, #e0f2fe 55%, #dbeafe 100%)",
          color: "#0f172a",
          fontFamily: "Inter, ui-sans-serif, system-ui",
          position: "relative",
          overflow: "hidden",
        }}
      >
        <div
          style={{
            position: "absolute",
            right: -90,
            top: -120,
            width: 460,
            height: 460,
            borderRadius: "9999px",
            background:
              "radial-gradient(circle, rgba(14,165,233,0.2) 0%, rgba(14,165,233,0) 72%)",
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
              letterSpacing: "0.2em",
              textTransform: "uppercase",
              color: "#0369a1",
              fontWeight: 700,
            }}
          >
            <span
              style={{
                width: 12,
                height: 12,
                borderRadius: 9999,
                background: "#0284c7",
                display: "flex",
              }}
            />
            Portal Docs
          </div>

          <div
            style={{
              display: "flex",
              flexDirection: "column",
              fontSize: 84,
              lineHeight: 0.96,
              letterSpacing: "-0.05em",
              fontWeight: 800,
              color: "#0f172a",
            }}
          >
            <span>Install, transfer,</span>
            <span>troubleshoot.</span>
          </div>

          <p
            style={{
              margin: 0,
              marginTop: 8,
              maxWidth: 850,
              fontSize: 32,
              lineHeight: 1.35,
              color: "#334155",
            }}
          >
            Practical command guides for Portal's production CLI workflow.
          </p>
        </div>

        <div
          style={{
            display: "flex",
            justifyContent: "space-between",
            alignItems: "center",
            borderTop: "1px solid rgba(2,132,199,0.25)",
            paddingTop: 26,
            fontSize: 24,
            color: "#475569",
          }}
        >
          <span style={{ display: "flex" }}>portal.build.app/docs</span>
          <span
            style={{
              display: "flex",
              padding: "8px 16px",
              borderRadius: 9999,
              border: "1px solid rgba(2,132,199,0.45)",
              color: "#075985",
              fontSize: 20,
              letterSpacing: "0.08em",
              textTransform: "uppercase",
              fontWeight: 700,
            }}
          >
            Documentation
          </span>
        </div>
      </div>
    ),
    size,
  );
}
