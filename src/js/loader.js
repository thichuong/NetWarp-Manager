// Component loader utility to fetch and inject HTML templates into the DOM
// All comments in English as per user rules

export async function loadComponents() {
  const components = [
    { id: "header-container", path: "/components/header.html" },
    { id: "speed-wifi-container", path: "/components/speed_wifi_section.html" },
    { id: "warp-control-container", path: "/components/warp_control_section.html" },
    { id: "modals-container", path: "/components/wifi_modal.html", append: true },
    { id: "modals-container", path: "/components/password_modal.html", append: true },
    { id: "modals-container", path: "/components/toast.html", append: true },
    { id: "footer-container", path: "/components/footer.html" }
  ];

  await Promise.all(
    components.map(async (comp) => {
      try {
        const response = await fetch(comp.path);
        if (!response.ok) {
          throw new Error(`Failed to load ${comp.path}: ${response.statusText}`);
        }
        const html = await response.text();
        const container = document.getElementById(comp.id);
        if (container) {
          if (comp.append) {
            container.insertAdjacentHTML("beforeend", html);
          } else {
            container.innerHTML = html;
          }
        } else {
          console.warn(`[Loader] Target container #${comp.id} not found in the DOM.`);
        }
      } catch (error) {
        console.error(`[Loader] Error loading component from ${comp.path}:`, error);
      }
    })
  );
}
