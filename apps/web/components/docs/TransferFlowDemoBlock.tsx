"use client";

import dynamic from "next/dynamic";

const TransferFlowDemo = dynamic(
  () => import("@/components/docs/TransferFlowDemo"),
  {
    ssr: false,
  },
);

export default function TransferFlowDemoBlock() {
  return <TransferFlowDemo />;
}
