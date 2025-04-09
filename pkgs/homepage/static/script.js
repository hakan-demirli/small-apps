document.addEventListener("DOMContentLoaded", () => {
  const tabBar = document.getElementById("tab-bar");
  const gridContainer = document.getElementById("grid-container");

  let state = {
    tabs: [],
    activeTab: 0,
  };

  // --- API Communication ---
  async function fetchState() {
    try {
      const response = await fetch("/api/state");
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      state = await response.json();
      // Ensure activeTab is valid
      if (state.activeTab >= state.tabs.length || state.activeTab < 0) {
        state.activeTab = 0;
      }
      // Ensure items array exists for all tabs
      state.tabs.forEach((tab) => {
        if (!Array.isArray(tab.items)) {
          tab.items = [];
        }
      });

      renderUI();
    } catch (error) {
      console.error("Failed to fetch state:", error);
      // Initialize with default if fetch fails critically
      state = { tabs: [{ name: "Default", items: [] }], activeTab: 0 };
      renderUI();
      alert("Could not load state from server. Using default.");
    }
  }

  async function saveState() {
    try {
      const response = await fetch("/api/state", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(state),
      });
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      // console.log("State saved successfully"); // Optional debug
    } catch (error) {
      console.error("Failed to save state:", error);
      alert("Error saving changes to the server.");
    }
  }

  // --- Rendering ---
  function renderUI() {
    renderTabs();
    renderGrid();
  }

  function renderTabs() {
    tabBar.innerHTML = ""; // Clear existing tabs

    state.tabs.forEach((tab, index) => {
      const tabButton = document.createElement("button");
      tabButton.className = "tab-button";
      tabButton.textContent = tab.name;
      if (index === state.activeTab) {
        tabButton.classList.add("active");
      }
      tabButton.addEventListener("click", () => {
        state.activeTab = index;
        renderUI(); // Re-render both tabs (for active class) and grid
        saveState(); // Save the change in active tab
      });
      tabBar.appendChild(tabButton);
    });

    // Add '+' button for adding new tabs
    const addTabButton = document.createElement("button");
    addTabButton.className = "add-tab-button";
    addTabButton.textContent = "+";
    addTabButton.title = "Add new tab";
    addTabButton.addEventListener("click", handleAddTab);
    tabBar.appendChild(addTabButton);
  }

  function renderGrid() {
    gridContainer.innerHTML = ""; // Clear existing grid items

    // Handle cases with no tabs
    if (
      state.tabs.length === 0 ||
      state.activeTab < 0 ||
      state.activeTab >= state.tabs.length
    ) {
      // Optionally display a message or just show the add item button if needed
      // gridContainer.innerHTML = '<p style="color: var(--comment-color); grid-column: 1 / -1;">No tabs available. Add one using the + button above.</p>';
      // If you want the add item button even with no tabs (might be confusing):
      // createAndAppendAddItemButton();
      return; // Exit rendering if no valid tab is active
    }

    const currentTab = state.tabs[state.activeTab];
    // Ensure items is always an array
    const items = Array.isArray(currentTab.items) ? currentTab.items : [];

    items.forEach((item) => {
      const gridLink = document.createElement("a");
      gridLink.className = "grid-item glass-effect"; // Added glass-effect here
      gridLink.href = item.url;
      // gridLink.target = "_blank"; // Open in new tab
      gridLink.rel = "noopener noreferrer"; // Security best practice

      const nameSpan = document.createElement("span");
      nameSpan.textContent = item.name;
      gridLink.appendChild(nameSpan);

      gridContainer.appendChild(gridLink);
    });

    // Add '+' button for adding new items
    createAndAppendAddItemButton();
  }

  function createAndAppendAddItemButton() {
    const addItemButton = document.createElement("button");
    addItemButton.className = "add-item-button glass-effect"; // Added glass-effect here
    addItemButton.textContent = "+";
    addItemButton.title = "Add new link";
    addItemButton.addEventListener("click", handleAddItem);
    gridContainer.appendChild(addItemButton);
  }

  // --- Event Handlers ---
  function handleAddTab() {
    const newTabName = prompt("Enter name for the new tab:");
    if (newTabName && newTabName.trim() !== "") {
      state.tabs.push({ name: newTabName.trim(), items: [] });
      state.activeTab = state.tabs.length - 1; // Activate the new tab
      renderUI();
      saveState();
      // Scroll the new tab into view if tab-bar is scrollable
      const tabButtons = tabBar.querySelectorAll(".tab-button");
      if (tabButtons.length > 0) {
        tabButtons[tabButtons.length - 1].scrollIntoView({
          behavior: "smooth",
          inline: "nearest",
        });
      }
    } else if (newTabName !== null) {
      // Alert only if not cancelled
      alert("Tab name cannot be empty.");
    }
  }

  function handleAddItem() {
    if (state.tabs.length === 0) {
      alert("Please add a tab first!");
      return;
    }
    const url = prompt("Enter the URL (e.g., https://example.com):");
    if (url && url.trim() !== "") {
      const name = prompt("Enter a name for this link:");
      if (name && name.trim() !== "") {
        // Ensure the items array exists
        if (!Array.isArray(state.tabs[state.activeTab].items)) {
          state.tabs[state.activeTab].items = [];
        }
        // Basic URL validation (optional but good)
        let finalUrl = url.trim();
        if (
          !finalUrl.startsWith("http://") &&
          !finalUrl.startsWith("https://") &&
          !finalUrl.startsWith("//")
        ) {
          finalUrl = "http://" + finalUrl; // Add http if missing (simplistic)
        }

        state.tabs[state.activeTab].items.push({
          name: name.trim(),
          url: finalUrl,
        });
        renderGrid(); // Only need to re-render the grid
        saveState();
      } else if (name !== null) {
        alert("Link name cannot be empty.");
      }
    } else if (url !== null) {
      alert("URL cannot be empty.");
    }
  }

  // --- Keyboard Shortcuts ---
  const tabShortcuts = {
    u: 0, // Alt+U -> First tab (index 0)
    i: 1, // Alt+I -> Second tab (index 1)
    o: 2, // Alt+O -> Third tab (index 2)
    p: 3, // Alt+P -> Fourth tab (index 3)
    // Add more mappings here if needed, e.g., 'k': 4
  };

  document.addEventListener("keydown", (event) => {
    // We only care about Alt + key combinations
    if (!event.altKey) {
      return;
    }

    // Prevent shortcuts from triggering if user is typing in a prompt or future input field
    if (
      event.target.tagName === "INPUT" ||
      event.target.tagName === "TEXTAREA"
    ) {
      // Although prompts block execution, this is good practice for potential future UI elements
      return;
    }

    const key = event.key.toLowerCase(); // Use lowercase for case-insensitivity

    if (tabShortcuts.hasOwnProperty(key)) {
      const targetTabIndex = tabShortcuts[key];

      // Check if the target tab exists in the current state
      if (targetTabIndex >= 0 && targetTabIndex < state.tabs.length) {
        // Prevent the default browser action associated with the shortcut (if any)
        event.preventDefault();

        // Check if we are actually changing the tab
        if (state.activeTab !== targetTabIndex) {
          state.activeTab = targetTabIndex;
          renderUI(); // Re-render tabs and grid
          saveState(); // Save the new active tab index

          // Optional: Scroll the newly activated tab into view if needed
          const tabButtons = tabBar.querySelectorAll(".tab-button");
          if (tabButtons[targetTabIndex]) {
            tabButtons[targetTabIndex].scrollIntoView({
              behavior: "smooth",
              inline: "nearest",
            });
          }
        }
      }
      // Optional: Provide feedback if the shortcut corresponds to a non-existent tab
      // else {
      //    console.log(`Shortcut Alt+${key.toUpperCase()} pressed, but tab index ${targetTabIndex} does not exist.`);
      // }
    }
  });

  // --- Initial Load ---
  fetchState();
}); // End of DOMContentLoaded listener
