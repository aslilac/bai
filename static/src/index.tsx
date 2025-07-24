import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { createBrowserRouter, RouterProvider } from "react-router";

function App() {
	return <h1>hello, computer!</h1>;
}

const appRoot = createRoot(document.getElementById("app")!);

const router = createBrowserRouter([
	{
		path: "/",
		element: <App />,
	},
]);

appRoot.render(
	<StrictMode>
		<RouterProvider router={router} />
	</StrictMode>,
);
