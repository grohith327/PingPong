import { serve } from "bun";

const server = serve({
  port: 3000,
  fetch: async (req) => {
    const response = {
      ping: "pong"
    };
    if (req.method == "GET") {
      return new Response(JSON.stringify({...response, message: "Successful GET request"}), {status: 200});
    }

    if (req.method == "POST") {
      const body = await req.json();
      console.log(`Request Body: ${JSON.stringify(body)}`);
      return new Response(JSON.stringify({...response, message: "Successful POST request"}), {status: 200});
    }
  }
});

console.log("Server started on localhost:3000");