import { serve } from "@hono/node-server";
import { Hono } from "hono";

const app = new Hono();
app.get("/healthz", (c) => c.body(null, 204));
app.get("/internal/v1/supported", (c) => c.json({
  x402Version: 2, schemes: [{ scheme: "upto", networks: ["eip155:84532", "eip155:8453"] }],
  verificationConfigured: false, settlementConfigured: false,
}));
app.post("/internal/v1/verify", (c) => c.json({ code: "payment_edge_unconfigured" }, 503));
app.post("/internal/v1/settle", (c) => c.json({ code: "payment_edge_unconfigured" }, 503));

serve({ fetch: app.fetch, port: Number(process.env.PORT ?? 8081), hostname: "127.0.0.1" });

