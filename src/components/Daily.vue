<template>
  <div class="daily">
    <h1>{{date}}</h1>
    <div v-for='entry in entries'>
      <h2>{{entry.member.name}}</h2>
      <h3>昨日したこと</h3>
      <ul id='yesterday'>
        <li v-for='work in entry.done'>{{work.title}}</li>
      </ul>
      <h3>今日すること</h3>
      <ul id='today'>
      <li v-for='work in entry.to_do'>{{work.title}}</li>
      </ul>
      <h3>障害/業務外の予定</h3>
      <ul id='problem'>
      <li v-for='issue in entry.problem'>{{issue.title}}</li>
      </ul>
    </div>
  </div>
</template>

<script>
import axios from 'axios'
export default {
  name: 'daily',
  data () {
    return {
      date: (new Date()).toLocaleDateString(),
      entries: []
    }
  },
  mounted () {
    axios.get('data/').then(res => {
      this.date = res.data.date
      this.entries = res.data.entries
    }).catch(err => {
      console.error(err)
    })
  }
}
</script>

<style scoped>
h3 {
  margin: 0 .5em;
}
ul {
  list-style-type: none;
  padding: 0;
}

li {
  display: inline-block;
  margin: 0 1em;
}
</style>
