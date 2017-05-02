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
            <span v-on:click.capture="deleteYesterdayWork($event)" v-on:change="updateYesterdayWork($event)">
            <li v-for="(work, index) in entries.yesterday">
              <div :data-index="index"><input v-bind:value="work.title"/><button>-</button></div>
            </li>
            </span>
            <li>
              <button v-on:click="addYesterdayWork">+</button>
            </li>
          </ul>
        </li>
        <li class="issue">
          <h3>今日すること</h3>
          <ul id='today' >
            <span v-on:click.capture="deleteTodayWork($event)" v-on:change="updateTodayWork($event)">
              <li v-for="(work, index) in entries.today">
                <div :data-index="index"><input v-bind:value="work.title"/><button>-</button></div>
              </li>
            </span>
            <li>
              <button v-on:click="addTodayWork">+</button>
            </li>
          </ul>
        </li>
        <li class="issue">
          <h3>障害/業務外の予定</h3>
          <ul id='problem'>
            <span v-on:click.capture="deleteIssue($event)" v-on:change="updateIssue($event)">
            <li v-for="(issue, index) in entries.issue">
              <div :data-index="index"><input v-bind:value="issue.title"/><button>-</button></div>
            </li>
            </span>
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
  import axios from 'axios'
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
    mounted () {
      this.loadData()
    },
    methods: {
      dateClicked (ev) {
        this.today = parseInt(ev.target.textContent)
        this.loadData()
      },
      dateStr () {
        let ds = '' + this.year + ('0' + (this.month + 1)).substr(-2) + ('0' + this.today).substr(-2)
        return ds
      },
      loadData () {
        axios.get('/entry/' + this.dateStr()).then(res => {
          this.entries.yesterday = res.data.done
          this.entries.today = res.data.to_do
          this.entries.issue = res.data.problem
        }).catch(err => {
          this.entries.yesterday = []
          this.entries.today = []
          this.entries.issue = []
          console.error(err)
        })
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
        this.entries.yesterday.push({title: ''})
      },
      updateYesterdayWork (work) {
        let index = work.target.parentNode.dataset.index
        this.entries.yesterday[index].title = work.target.value
        axios.post('/entry/' + this.dateStr() + '/done', this.entries.yesterday).then(res => {
        }).catch(err => {
          console.error(err)
        })
      },
      deleteYesterdayWork (work) {
        if (work.target instanceof HTMLButtonElement) {
          let index = work.target.parentNode.dataset.index
          this.entries.yesterday.splice(index, 1)
        }
      },
      addTodayWork () {
        this.entries.today.push({title: ''})
      },
      updateTodayWork (work) {
        let index = work.target.parentNode.dataset.index
        this.entries.today[index].title = work.target.value
        axios.post('/entry/' + this.dateStr() + '/todo', this.entries.today).then(res => {
        }).catch(err => {
          console.error(err)
        })
      },
      deleteTodayWork (work) {
        if (work.target instanceof HTMLButtonElement) {
          let index = work.target.parentNode.dataset.index
          this.entries.today.splice(index, 1)
        }
      },
      addIssue () {
        this.entries.issue.push({title: ''})
      },
      updateIssue (work) {
        let index = work.target.parentNode.dataset.index
        this.entries.issue[index].title = work.target.value
        axios.post('/entry/' + this.dateStr() + '/problem', this.entries.issue).then(res => {
        }).catch(err => {
          console.error(err)
        })
      },
      deleteIssue (work) {
        if (work.target instanceof HTMLButtonElement) {
          let index = work.target.parentNode.dataset.index
          this.entries.issue.splice(index, 1)
        }
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
