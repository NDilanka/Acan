const todos = require('../model/todo');

function legacyTitles(list) {
  return list.map((todo) => todo.title);
}

module.exports = { legacyTitles, todos };
