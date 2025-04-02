import { serve } from "bun";

const server = serve({
  port: 3000,
  fetch: async (req) => {
    const response = {
      ping: "pong",
    };
    if (req.method == "GET") {
      return new Response(
        JSON.stringify({ ...response, message: "Successful GET request" }),
        { status: 200 },
      );
    }

    if (req.method == "POST") {
      const body = await req.json();
      const bodyJson = JSON.stringify(body);
      console.log(`POST Request Body: ${bodyJson}`);
      console.log(`POST Request headers: ${JSON.stringify(req.headers)}`);
      if (body.hello === "world") {
        return new Response(
          JSON.stringify({ ...response, message: "Successful POST request" }),
          { status: 200 },
        );
      } else {
        return new Response(
          JSON.stringify({ errorMessage: "Invalid Request" }, { status: 400 }),
        );
      }
    }

    if (req.method == "PATCH") {
      const body = await req.json();
      console.log(`PATCH Request Body: ${JSON.stringify(body)}`);
      return new Response(
        JSON.stringify({ ...response, message: "Successful PATCH request" }),
        { status: 200 },
      );
    }

    if (req.method == "PUT") {
      const body = await req.json();
      console.log(`PUT Request Body: ${JSON.stringify(body)}`);
      return new Response(
        JSON.stringify({ ...response, message: "Successful PUT request" }),
        { status: 200 },
      );
    }

    if (req.method == "DELETE") {
      const body = await req.json();
      console.log(`DELETE Request Body: ${JSON.stringify(body)}`);
      return new Response(
        JSON.stringify({ ...response, message: "Successful DELETE request" }),
        { status: 200 },
      );
    }
  },
});

console.log("Server started on localhost:3000");
