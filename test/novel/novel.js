exports.Book = class {
  constructor(prop) {
    this.title = prop.title;
    this.leftPage = prop.left_page;
    this.size = prop.size;
  }

  turnPage(count) {
    this.leftPage += 2 * count;

    if (this.leftPage < 0) {
      this.leftPage = 0;
    } else if (this.leftPage > this.size) {
      this.leftPage = this.size - 1;
    }
  }

  getPage() {
    return this.leftPage;
  }
};
