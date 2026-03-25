"use client";

import dynamic from "next/dynamic";

const ConfigSetupDemo = dynamic(
  () => import("@/components/docs/ConfigSetupDemo"),
  {
    ssr: false,
  },
);

export default function ConfigSetupDemoBlock() {
  return <ConfigSetupDemo />;
}
