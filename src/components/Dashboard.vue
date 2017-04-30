<template>
  <div class="dashboard">
    <table class="calender">
      <caption>{{year}}/{{month + 1}}</caption>
      <thead>
      <th>Sun</th>
      <th>Mon</th>
      <th>Tue</th>
      <th>Wed</th>
      <th>Thu</th>
      <th>Fri</th>
      <th>Sat</th>
      </thead>
      <tbody v-on:click.capture="dateClicked($event)">
      <tr v-for="week in weeks"><td v-for="day in week" v-bind:class="calendarClass(day)">{{!day ? '' : day.day}}</td></tr>
      </tbody>
    </table>
    <div class="entry">
      <h1>{{year}}/{{month + 1}}/{{today}}</h1>
      <ul>
        <li class="issue">
          <h3>昨日したこと</h3>
          <ul id='yesterday'>
            <li v-for="(work, index) in entries.yesterday"><input v-bind:value="work" v-on:change="updateYesterdayWork($event, index)"/></li>
            <li>
              <button v-on:click="addYesterdayWork">+</button>
            </li>
          </ul>
        </li>
        <li class="issue">
          <h3>今日すること</h3>
          <ul id='today' >
            <li v-for="(work, index) in entries.today"><input v-bind:value="work" v-on:change="updateTodayWork($event, index)"/></li>
            <li>
              <button v-on:click="addTodayWork">+</button>
            </li>
          </ul>
        </li>
        <li class="issue">
          <h3>障害/業務外の予定</h3>
          <ul id='problem'>
            <li v-for="(issue, index) in entries.issue"><input v-bind:value="issue" v-on:change="updateIssue($event, index)"/></li>
            <li>
              <button v-on:click="addIssue">+</button>
            </li>
          </ul>
        </li>
      </ul>
    </div>
  </div>
</template>

<script>
  export default {
    name: 'dashboard',
    data () {
      return {
        date: '2017/4/29',
        year: 0,
        month: 0,
        today: 0,
        weeks: [],
        entries: {
          yesterday: [],
          today: [],
          issue: []
        }
      }
    },
    created () {
      let dv = new Date(Date.parse(this.date))
      this.year = dv.getFullYear()
      this.month = dv.getMonth()
      this.today = dv.getDate()
      let startDay = new Date(this.year, this.month, 1, 9)
      let endDay = new Date(this.year, this.month + 1, 0, 9)
      let wd = startDay.getDay()
      var days = []
      for (let i = 1; i <= endDay.getDate(); wd++, i++) {
        days[wd] = {day: i, wd: wd}
        if (wd === 6) {
          this.weeks.push(days)
          days = []
          wd = -1
        }
      }
      if (days.length > 0) {
        this.weeks.push(days)
      }
    },
    methods: {
      dateClicked (ev) {
        this.today = parseInt(ev.target.textContent)
        this.entries.yesterday = []
        this.entries.today = []
        this.entries.issue = []
      },
      calendarClass (day) {
        let ret = {
          today: false,
          saturday: false,
          sunday: false
        }
        if (day === undefined) {
          return ret
        }
        if (day.wd === 0) {
          ret.sunday = true
        } else if (day.wd === 6) {
          ret.saturday = true
        }
        if (day.day === this.today) {
          ret.today = true
        }
        return ret
      },
      addYesterdayWork () {
        this.entries.yesterday.push('')
      },
      updateYesterdayWork (work, index) {
        this.entries.yesterday[index] = work.target.value
      },
      addTodayWork () {
        this.entries.today.push('')
      },
      updateTodayWork (work, index) {
        this.entries.today[index] = work.target.value
      },
      addIssue () {
        this.entries.issue.push('')
      },
      updateIssue (work, index) {
        this.entries.issue[index] = work.target.value
      }
    }
  }
</script>

<style scoped>
  .calender caption {
    font-size: 1.2em;
  }
  .calender th {
    text-align: center;
    width: 2em;
  }

  .calender td {
    text-align: center;
  }
  .calender td.sunday {
    color: red;
  }
  .calender td.saturday {
    color: blue;
  }
  .calender td.today {
    color: white;
    background-color: skyblue;
  }
  .calender td.today.saturday {
    background-color: blue;
  }
  .calender td.today.sunday {
    background-color: red;
  }

  .entry>ul {
    list-style-type: none;
    padding: 0;
  }
  .issue ul {
    margin: 0 1em;
  }
  .issue li{
    display: inline-block;
    list-style-type: none;
    margin: .33em;
  }

</style>
