document.getElementById("submit").onclick = () => {
    document.getElementById("send").hidden = false;
  
    let rep = 0;
    setInterval(() => {
      if (rep <= 16) document.getElementById("send").innerText = `Sending Email [=${'='.repeat(rep)}>]`;
      else document.getElementById("send").innerText = `This is takeing a while...`;
      rep++;
    }, 250);
  };
  