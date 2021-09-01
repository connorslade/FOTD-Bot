// Define Questions to ask
let questions = [
  "ok but are you really really sure???",
  "really really sure?",
  ":'( it would make me sad",
];

// Add Event Listener
document.getElementById("doit").onclick = () => {
  if (localStorage.getItem("nextTime") > new Date().valueOf()) {
    alert("You can only try to Unsubscribe one every 24 hours.\n:)");
    return;
  }

  for (let i = 0; i < questions.length; i++) {
    if (!confirm(questions[i])) {
      alert("Good Choice! : )");
      setUnsubLock();
      break;
    }
    if (i == questions.length - 1) {
      alert("fine...");
      setUnsubLock();
      window.location.href = "/unsubscribe/real";
    }
  }
};

function setUnsubLock() {
  let nextTime = new Date().valueOf() + 24 * 60 * 60;
  localStorage.setItem("nextTime", new Date(nextTime * 1000).valueOf());
}
