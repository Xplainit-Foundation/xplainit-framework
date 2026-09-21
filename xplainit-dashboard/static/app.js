// Xplainit Dashboard frontend (vanilla JS, no build step).
//
// Connects to the SSE stream at /events for the live/replay timeline, and
// fetches /summary and /callgraph for the aggregated error view and the call
// graph tree. Everything renders with plain DOM APIs so it runs in any browser
// without a bundler or npm toolchain.

(function () {
  "use strict";

  var connDot = document.getElementById("conn-dot");
  var connLabel = document.getElementById("conn-label");
  var timelineEl = document.getElementById("timeline");

  function setStatus(state, label) {
    connDot.className = "dot dot-" + state;
    connLabel.textContent = label;
  }

  function shortTime(iso) {
    // Show just the time portion when possible.
    var t = iso.indexOf("T");
    if (t === -1) {
      return iso;
    }
    return iso.slice(t + 1).replace("Z", "");
  }

  function addTimelineEntry(evt) {
    var li = document.createElement("li");
    if (evt.is_error) {
      li.className = "error";
    }

    var ts = document.createElement("span");
    ts.className = "ts";
    ts.textContent = shortTime(evt.timestamp || "");

    var type = document.createElement("span");
    type.className = "type";
    type.textContent = evt.event_type;

    var loc = document.createElement("span");
    loc.className = "loc";
    if (evt.location && evt.location.file) {
      loc.textContent = evt.location.file + ":" + evt.location.line;
    }

    li.appendChild(ts);
    li.appendChild(type);
    li.appendChild(loc);
    timelineEl.appendChild(li);
    li.scrollIntoView({ block: "nearest" });
  }

  function renderSummary(summary) {
    document.getElementById("total-events").textContent = summary.total_events;
    document.getElementById("error-count").textContent = summary.error_count;

    var tbody = document.getElementById("error-counts");
    tbody.innerHTML = "";
    var counts = summary.counts_by_type || {};
    Object.keys(counts).forEach(function (key) {
      var tr = document.createElement("tr");
      var name = document.createElement("td");
      name.textContent = key;
      var count = document.createElement("td");
      count.textContent = counts[key];
      tr.appendChild(name);
      tr.appendChild(count);
      tbody.appendChild(tr);
    });

    var analysesEl = document.getElementById("analyses");
    analysesEl.innerHTML = "";
    (summary.analyses || []).forEach(function (a) {
      var box = document.createElement("div");
      box.className = "analysis";

      var title = document.createElement("h3");
      title.textContent =
        (a.error_type || a.event_type) + " (" + a.severity + ")";
      box.appendChild(title);

      var cause = document.createElement("div");
      cause.textContent = a.root_cause;
      box.appendChild(cause);

      if (a.fix_suggestions && a.fix_suggestions.length) {
        var ul = document.createElement("ul");
        a.fix_suggestions.forEach(function (fix) {
          var item = document.createElement("li");
          item.textContent = fix;
          ul.appendChild(item);
        });
        box.appendChild(ul);
      }
      analysesEl.appendChild(box);
    });
  }

  function renderCallGraph(graph) {
    var container = document.getElementById("call-graph");
    container.innerHTML = "";
    (graph.nodes || []).forEach(function (node) {
      var row = document.createElement("div");
      row.className = "call-node";
      var indent = new Array(node.depth + 1).join("  ");
      var branch = node.depth > 0 ? "\u2514\u2500 " : "";
      var label = document.createElement("span");
      label.className = "fn";
      label.textContent = node.name + "()";
      row.appendChild(document.createTextNode(indent + branch));
      row.appendChild(label);
      container.appendChild(row);
    });
    if (!graph.nodes || graph.nodes.length === 0) {
      container.textContent = "No function calls in this trace.";
    }
  }

  function renderTasks(view) {
    var panel = document.getElementById("tasks-panel");
    var container = document.getElementById("tasks");
    container.innerHTML = "";
    var tasks = (view && view.tasks) || [];
    if (tasks.length === 0) {
      // No async tasks in this trace: keep the panel hidden to avoid clutter.
      panel.hidden = true;
      return;
    }
    panel.hidden = false;

    tasks.forEach(function (task) {
      var box = document.createElement("div");
      box.className = "task";

      var title = document.createElement("h3");
      var name = task.task_name || task.task_id;
      title.textContent = name + " [" + task.state + "]";
      box.appendChild(title);

      var meta = document.createElement("div");
      meta.className = "task-meta";
      meta.textContent = task.event_count + " lifecycle event(s)";
      box.appendChild(meta);

      var ul = document.createElement("ul");
      (task.events || []).forEach(function (evt) {
        var li = document.createElement("li");
        var loc = "";
        if (evt.location && evt.location.file) {
          loc = " (" + evt.location.file + ":" + evt.location.line + ")";
        }
        li.textContent = shortTime(evt.timestamp || "") + " " + evt.event_type + loc;
        ul.appendChild(li);
      });
      box.appendChild(ul);
      container.appendChild(box);
    });
  }

  function loadJson(url, onOk) {
    fetch(url)
      .then(function (res) {
        return res.json();
      })
      .then(onOk)
      .catch(function (err) {
        console.error("failed to load " + url, err);
      });
  }

  function startStream() {
    if (typeof EventSource === "undefined") {
      setStatus("closed", "SSE unsupported in this browser");
      return;
    }
    var source = new EventSource("/events");
    source.onopen = function () {
      setStatus("live", "streaming");
    };
    source.onmessage = function (message) {
      try {
        var evt = JSON.parse(message.data);
        // The server sends a terminal {"done":true} sentinel to tell the client
        // the trace is exhausted. It is not a real event, so skip it rather than
        // rendering a blank timeline row.
        if (evt && evt.done) {
          return;
        }
        addTimelineEntry(evt);
      } catch (e) {
        console.error("bad event payload", e);
      }
    };
    source.onerror = function () {
      // The server closes the stream when the trace is exhausted.
      setStatus("closed", "stream ended");
      source.close();
    };
  }

  loadJson("/summary", renderSummary);
  loadJson("/callgraph", renderCallGraph);
  loadJson("/tasks", renderTasks);
  startStream();
})();
