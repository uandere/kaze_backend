//////////////////////////////////////////////////
//                   FUNCTIONS                  //
//////////////////////////////////////////////////
#let currency_dative_case(currency: str) = [
  #if currency == "USD" {
    return "доларів США"
  } else if currency == "EUR" {
    return "євро"
  } else {
    return "гривень"
  }
]

#let currency_dative_case(currency: str) = [
  #if currency == "USD" {
    return "доларів США"
  } else if currency == "EUR" {
    return "євро"
  } else {
    return "гривень"
  }
]

#let currency_genitive_case(currency: str) = [
  #if currency == "USD" {
    return "долара США"
  } else if currency == "EUR" {
    return "євро"
  } else {
    return "гривні"
  }
]

//////////////////////////////////////////////////
//                   VARIABLES                  //
//////////////////////////////////////////////////
 
#let font-size = (
  text: 12pt,
  heading: 14pt
)
 
//////////////////////////////////////////////////
//                   DOCUMENT                   //
//////////////////////////////////////////////////
 
#set document(
  title: "Договір оренди нерухомого майна",
  date: datetime(
    day: 10,
    month: 8,
    year: 2024
  )
)
 
 
//////////////////////////////////////////////////
//                     PAGE                     //
//////////////////////////////////////////////////
 
#set page(
  paper: "a4",
  margin: (
    x: 1.8cm,
    y: 1.5cm
  ),
  numbering: "1",
)
 
 
//////////////////////////////////////////////////
//                     TEXT                     //
//////////////////////////////////////////////////
 
#set text(
  size: font-size.text,
  lang: "uk",
  region: "ua"
)
 
 
//////////////////////////////////////////////////
//                  PARAGRAPH                   //
//////////////////////////////////////////////////
 
#set par(
  leading: 1.1em,
  justify: true,
  spacing: 2em
)
 
 
//////////////////////////////////////////////////
//                   HEADINGS                   //
//////////////////////////////////////////////////
 
#set heading(
  numbering: "1.",
)
 
#show heading: set text(
  size: font-size.text,
  weight: "regular"
)
 
#show heading.where(level: 1): set text(
  size: font-size.heading,
  weight: "bold"
)
#show heading.where(level: 1): set block(
  above: 2em,
  below: 1em
)
#show heading.where(level: 1): set align(center)
 
 
//////////////////////////////////////////////////
//                    LISTS                     //
//////////////////////////////////////////////////
 
#show enum.item: it => {
  context {
    counter(heading).step(
      level: query(
        selector(heading)
        .before(here())
      ).at(-1).level + 1
    )
  }
  show grid: set block(
    above: if enum.tight{
      par.leading
    } else {
      par.spacing
    },
  )
  grid(
    columns: 2,
    gutter: enum.body-indent,
    context {
        counter(heading).display()
    },
    text(it.body)
  )
}


//////////////////////////////////////////////////
//                   TEMPLATE                   //
//////////////////////////////////////////////////
#let rental_agreement_title(
  rental_agreement_number: int
) = align(
  center,
  text(
    size: 17pt,
    weight: "bold",
    [#context document.title №#rental_agreement_number]
  )
)
 
#let rental_agreement_place_and_date(
  place: str,
  date: datetime
) = {
  grid(
    columns: (1fr, 1fr),
    align: (left, right),
    place,
    if date != datetime {
      date.display()
    } else {
      context document.date.display()
    }
  )
}
 
#let sides_of_agreement(
  tenant: dictionary,
  landlord: dictionary
) = [
  *Орендодавець:* #landlord.initials, який проживає за адресою: #landlord.address_of_residence, та має наступні паспортні дані: cерія: #landlord.passport_data.series, номер: #landlord.passport_data.number, виданий державним органом: #landlord.passport_data.issuing_authority, надалі *Орендодавець*, з одного боку, і
 
  *Орендар:* #tenant.initials, який проживає за адресою: #tenant.address_of_residence, та має наступні паспортні дані: cерія: #tenant.passport_data.series, номер: #tenant.passport_data.number, виданий державним органом: #tenant.passport_data.issuing_authority, надалі *Орендар*, з іншого боку (разом *Сторони*), уклали цей Договір про наступне:
]
 
#let subject_of_agreement(
  real_estate_data: dictionary,
  ownership_record: dictionary
) = [
  = Предмет договору
  + Предметом цього договору є тимчасова здача в оренду на оплатній основі житлового приміщення (тип приміщення: #real_estate_data.type) яка належить на праві власності Орендодавцю. Житлове приміщення розташоване за адресою #real_estate_data.address, і має загальну площу #real_estate_data.area м.кв. Житлове приміщення здається в оренду з усіма невід’ємними технічними пристроями і з предметами домашньої обстановки, згідно з Додатком №1 до цього Договору (далі Об’єкт нерухомості). Право власності Орендодавця на вказаний Об’єкт нерухомості підтверджується: Договір купівлі-продажу #ownership_record.number від #ownership_record.date.display().
  + Орендодавець підтверджує, що даний Об’єкт нерухомості нікому раніше не проданий, не подарований, в спорі і під арештом не перебуває, і претензій з боку третіх осіб до нього не має.
]

 
#let rights_and_obligations(rental_payment_delay_limit: int) = [
  = Права та обов'язки сторін
 
  == Орендодавець відповідно до умов цього Договору зобов'язується:
    + Передати Орендарю Об’єкт нерухомості, зазначений в п.1.1 Договору, не пізніше 1-го (одного) дня з моменту підписання Договору по Акту прийому-передачі житлового приміщення (Додаток №1 до Договору), який підписується Сторонами, додається до Договору і є його невід’ємною частиною.
    + Надати Орендарю Об’єкт нерухомості, який є предметом цього Договору в придатному для використання за призначенням стані.
    + Забезпечити належний стан Об’єкту нерухомості, а також комунікацій, що відносяться до Об’єкту нерухомості.
    + Забезпечити користування Орендарем комунальними та іншими послугами, що відносяться до Об’єкту нерухомості.
    + Проводити за свій рахунок капітальний ремонт Об’єкту нерухомості.
    + Не вчиняти дій, які можуть перешкоджати Орендарю користуватися Об’єктом нерухомості.
 
  == Орендодавець має право:
    + Вимагати розірвання Договору та відшкодування збитків у разі, якщо Орендар використовує Об’єкт нерухомості не за призначенням або з порушенням умов Договору.

    + Вимагати розірвання Договору в разі, якщо Орендар прострочив оплату орендної плати та комунальних послуг на термін більше, ніж на #rental_payment_delay_limit днів.

    + Вимагати від Орендаря сплати неустойки в розмірі місячної орендної плати за весь час прострочення в разі, якщо Орендар після закінчення терміну дії Договору не передав Орендодавцю Об’єкт нерухомості згідно з Актом здачі житлового приміщення.

  == Орендар відповідно до умов цього Договору зобов'язується:

    + Прийняти Об’єкт нерухомості в терміни і на умовах, визначених Договором та використовувати лише для проживання в ньому фізичних осіб.

    + Забезпечувати збереження та утримання в належному стані Об’єкту нерухомості на умовах, визначених Договором, не допускаючи його псування або приведення в непридатність.

    + Дотримуватися Правил використання приміщень житлових будинків і прибудинкових територій.

    + Перевірити в присутності Орендодавця справність майна згідно з Додатком №1 до Договору.

    + Дотримуватися умов цього Договору щодо своєчасності та повноти внесення орендної плати та інших платежів.

    + Своєчасно повідомляти Орендодавця про несправності технічних пристроїв Об’єкту нерухомості.

    + Орендар зобов’язаний усунути погіршення Об’єкту нерухомості, що трапилося з його вини. При невиконанні зазначених зобов’язань Орендар відшкодовує Орендодавцю всі витрати і збитки, пов’язані з ремонтом Об’єкту нерухомості.

    + Звільнити і здати Орендодавцю Об’єкт нерухомості в належному стані з урахуванням нормального фізичного зносу протягом 1-ого (одного) дня з моменту закінчення терміну дії (розірвання) цього Договору, згідно з Актом здачі житлового приміщення (Додаток №2 до Договору). Орендар, який затримав здачу Орендодавцю об’єкта нерухомості, несе ризик та відповідальність за його випадкове знищення або випадкове пошкодження.

    + Надавати Орендодавцю за першою вимогою всю необхідну інформацію щодо орендованого Об’єкту нерухомості.

    == Орендар має право:

    + У разі, якщо Орендодавець не передав Об’єкт нерухомості в термін згідно п. 2.1.1 Договору, вимагати від Орендодавця передачі Об’єкту нерухомості і сплати неустойки в розмірі місячної орендної плати за весь час прострочення.

    + Орендар має переважне право перед іншими особами на укладення договору оренди на новий термін.
]

#let rental_payment(rental_payment_data: dictionary) = [
  = Орендна плата

  + За домовленістю Сторін щомісячна плата за користування Об’єктом нерухомості (орендна плата) вноситься в грошовій формі. Орендар за кожен місяць користування Об’єктом нерухомості оплачує Орендодавцю орендну плату. *Орендна плата становить #rental_payment_data.amount #currency_dative_case(currency: rental_payment_data.currency)*.

  #if rental_payment_data.currency != "UAH" {
    [
      + Сума щомісячної плати вираховується шляхом множення орендної плати на курс #currency_genitive_case(currency: rental_payment_data.currency) до гривні, який встановлений Національним Банком України, на момент здійснення платежу.
    ]
  }

  + Розрахунок за орендну плату виконується шляхом переводу грошей Орендарем на банківську картку Орендодавця. Номер картки Орендодавця: *#rental_payment_data.destination*.

  + Зміна Сторонами Договору, розміру орендної плати протягом терміну дії цього Договору, можливо тільки за згодою Сторін і закріплюється Додатковими угодами.

  + Нарахування орендної плати починається від дати #rental_payment_data.starting_date.display() і фактичного використання Об’єкту нерухомості згідно з Актом прийому-передачі житлового приміщення (Додатки №1 до Договору), підписаного обома Сторонами.

  + Оплата комунальних послуг, що відносяться до Об’єкту нерухомості, проводиться Орендарем. Також Орендарем сплачуються наступні послуги, що відносяться до Об’єкту нерухомості:
    - Інтернет Astra (щомісяця)
    - Домофон (щомісяця)

  + Розрахунок по орендній платі за Об’єкт нерухомості проводиться між Орендодавцем та Орендарем не пізніше #rental_payment_data.payment_day_number\-го числа кожного місяця проживання, що підлягає оплаті.

  + Повнота і своєчасність розрахунків по орендній платі, а також внесення інших платежів, передбачених Договором, підтверджується відповідними платіжними документами.

  == Особливості порядку розрахунків і депозитна сума за Об’єкт нерухомості: Орендар вносить депозитну суму в розмірі #rental_payment_data.amount #currency_dative_case(currency: rental_payment_data.currency) як гарантію виконання умов цього договору і збереження Об’єкту нерухомості і майна, зазначених у Акті прийому-передачі житлового приміщення (додаток №1 до Договору). Після закінчення терміну дії договору і звільнення Об’єкта нерухомості Орендарем, при необхідності Орендодавець може використовувати депозитну суму для усунення погіршення стану Об’єкта нерухомості, що трапилося з вини Орендаря, якщо це погіршення виходить за рамки природного зносу і амортизації Об’єкту нерухомості. Також, при необхідності, депозитна сума може бути використана для покриття комунальних платежів або інших витрат, відповідно до цього договору. Протягом семи днів після звільнення Об’єкта нерухомості Орендарем, Орендодавець повинен надати звіт про витрати і повернути суму, що залишилася Орендарю.
]

#let agreement_conditions(agreement_conditions_data: dictionary) = [
  = Термін дії, порядок продовження і розірвання Договору

  == Термін дії цього Договору встановлений з #agreement_conditions_data.starting_date.display() по #agreement_conditions_data.ending_date.display().

  === Моментом фактичного використання Об’єкту нерухомості за цим Договором є підписання Сторонами Акту прийому-передачі житлового приміщення (Додаток №1 до Договору).

  === Моментом закінчення фактичного використання Об’єкту нерухомості за цим Договором є підписання Сторонами Акту здачі житлового приміщення (Додаток №2 до Договору).

  === Підписанням відповідних актів підтверджується відсутність взаємних претензій і виконання Сторонами своїх зобов’язань за цим Договором.

  == Одностороння відмова від даного Договору не допускається, за винятком випадку, коли Об’єкт нерухомості став непридатним для подальшого використання за призначенням.

  ==  З ініціативи однієї із Сторін Договір може бути достроково розірваний, з письмовим повідомленням про це іншої Сторони не менше, ніж за 30 днів до передбачуваної дати розірвання Договору.

  == Договір припиняє свою дію у разі: закінчення терміну, на який його було укладено; знищення об’єкта нерухомості та в інших випадках, передбачених законодавством України.

  == Даний Договір вважається продовженим на той самий строк і на тих же умовах, в разі відсутності заяви однієї із Сторін про припинення або зміну умов Договору протягом одного місяця до закінчення терміну дії Договору.
]

#let responsibility() = [
  = Відповідальність Сторін

  == Відповідальність за справний технічний стан Об’єкта нерухомості на термін дії Договору несе Орендодавець.

  == Орендар відшкодовує Орендодавцю збитки, спричинені неналежним ремонтом або експлуатацією Об’єкта нерухомості та невід’ємного майна.

  == Орендар несе відповідальність за правильне і безпечне використання Об'єкту нерухомості.

  === Збиток, заподіяний Орендодавцю або третім особам внаслідок порушення Орендарем умов Договору, відшкодовується Орендарем самостійно і в повному обсязі.

  === При погіршенні стану або знищенні об’єкта нерухомості з вини Орендаря, він відшкодовує Орендодавцю збитки в розмірі вартості ремонту або відновлення майна.

  == Орендодавець відшкодовує збиток, заподіяний третім особам або Орендарю, якщо буде встановлено, що це сталося не з вини Орендаря або внаслідок особливих властивостей або недоліків, про наявність яких Орендаря не було попереджено Орендодавцем і про які він не знав і не міг знати.

  == За несвоєчасну оплату платежів за цим Договором Орендар сплачує на користь Орендодавця неустойку (пеню) в розмірі подвійної облікової ставки, встановленої Національним Банком України, від суми прострочених платежів за кожний день прострочення.

  == Спірні питання за цим Договором вирішуються в порядку, встановленому чинним законодавством України.

  == Сторони звільняються від якої б то не було відповідальності, якщо невиконання ними прийнятих на себе зобов’язань буде пов’язано з обставинами, що не залежать від їхньої волі, або бажання і знаходяться поза сферою контролю зобов’язаної сторони, але, при цьому, сторони повинні вжити всіх заходів до взаємного запобігання заподіяння майнових і фінансових втрат.
]

#let other_conditions(other_conditions_data: dictionary) = [
  = Інші умови

  + Справжнім Договором встановлюється, що Орендодавець (самостійно або за допомогою уповноваженого представника) має право відвідувати Об’єкт нерухомості з метою технічного огляду, профілактики та обслуговування об’єкта нерухомості тільки в присутності Орендаря і повинен попередити Орендаря про відвідування за #other_conditions_data.min_notice_days_for_visit дні.

  + Орендодавець має право відвідувати Об’єкт нерухомості без попередження Орендаря в разі надзвичайних ситуацій (аварії, пожежі, затоплення, протікання газу, тощо).

  + Орендар не має права проводити перевлаштування та реконструкцію Об’єкту нерухомості, змінювати стан майна без письмової згоди Орендодавця.

  + Фізичні особи, які будуть користуватися Об’єктом нерухомості для проживання:
    #list(..other_conditions_data.all_tenants)
  Зазначені особи набувають рівних з Орендарем прав і обов’язків щодо користування Об’єктом нерухомості.

  + Орендодавець не надає Орендарю право реєструвати (прописати) фізичних осіб за місцезнаходженням об’єкта нерухомості.

  + Орендар не має право укладати Договір суборенди на Об’єкт нерухомості.

  + Користуючись Об’єктом нерухомості Орендар має право утримувати тварин:
    #list(..other_conditions_data.allowed_animals)

  + Умови даного Договору зберігають свою силу на весь строк дії Договору, а також, якщо після його укладення законодавством встановлено правила, що погіршують становище Орендаря.

  + У разі продажу Об’єкта нерухомості, Орендар має переважне право перед іншими особами на його придбання.

  + Про зміну поштових, розрахунково-платіжних та інших реквізитів одна із Сторін зобов’язана повідомити іншу Сторону в 10-денний термін.

  + Сторони домовилися про те, що всі заяви, повідомлення, що стосуються даного Договору, повинні бути викладені в письмовому вигляді, і вважаються доведеними до відома відповідної Сторони, якщо вони відправлені по електронній пошті.

  + Цей Договір складений в двох примірниках, які мають однакову юридичну силу з відповідною кількістю Додатків і Додаткових угод, які є невід’ємною частиною даного Договору.

  + Зміна умов Договору здійснюється у письмовій формі за взаємною згодою Сторін.

  + Взаємовідносини Сторін, які не врегульовані цим Договором, регламентуються чинним законодавством України.

  + Сторонам відомо, що відповідно до Указу Президента України №64/2022 від 24.02.2022 року «Про введення воєнного стану в Україні», у зв’язку з військовою агресією Російської Федерації проти України запроваджено воєнний стан на всій території України. Сторони погоджуються, що запровадження воєнного стану відбулось до підписання Сторонами цього Договору тане впливає в майбутньому на виконання ними своїх зобов’язань.
]

#let signatures() = [
  #pagebreak()
  #heading(numbering: none)[Підписи сторін]

  #table(
    columns: 1,
    align: horizon,
    inset: 10pt,

    [
      *Орендодавець* #linebreak()
    ],
    [
      *Паспорт громадянина України*: Серія: -; Номер: 4323424322; виданий: 3344

      *Aдресa*: Україна, Волинська обл., м. Луцьк, Луцький р-н, с. Городище, вул. Дружби, 63
      
      *Телефон*: 0963211626
      
      *Email*: nazar.demchvk\@gmail.com
      
      *Демчук Назар Ігорович* \_\_\_\_\_\_\_\_\_\_ (Підпис)
    ],
  )

    #table(
    columns: 1,
    align: horizon,
    inset: 10pt,

    [
      *Орендар* #linebreak()
    ],
    [
      *Паспорт громадянина України*: Серія: -; Номер: 4323424322; виданий: 3344

      *Aдресa*: Україна, Волинська обл., м. Луцьк, Луцький р-н, с. Городище, вул. Дружби, 63
      
      *Телефон*: 0963211626
      
      *Email*: nazar.demchvk\@gmail.com
      
      *Демчук Назар Ігорович* \_\_\_\_\_\_\_\_\_\_ (Підпис)
    ],
  )
]


//////////////////////////////////////////////////
//                     BODY                     //
//////////////////////////////////////////////////
#rental_agreement_title(
  rental_agreement_number: 1
)
 
#rental_agreement_place_and_date(
  place: "Львів"
)
 
#sides_of_agreement(
  tenant: (
    initials: "Демчук Назар Ігорович",
    address_of_residence: "Україна, Волинська обл., м. Луцьк, Луцький р-н, с. Городище, вул. Дружби, 63",
    passport_data: (
      series: "-",
      number: "4323424322",
      issuing_authority: "3344",
    )
  ),
  landlord: (
    initials: "Скіра Володимир Васильович",
    address_of_residence: "Україна, Волинська обл., м. Луцьк, Луцький р-н, с. Городище, вул. Дружби, 63",
    passport_data: (
      series: "-",
      number: "5489939439",
      issuing_authority: "8754",
    )
  )
)
 
#subject_of_agreement(
  real_estate_data: (
    type: "квартира",
    address: "Україна, Львівська обл., м. Львів, Сихівський р-н, вул. Пимоненка 7к",
    area: 43
  ),
  ownership_record: (
    number: "34983948",
    date: datetime(
      day: 8,
      month: 11,
      year: 2023
    )
  )
)
 
#rights_and_obligations(
  rental_payment_delay_limit: 10
)

#rental_payment(
  rental_payment_data: (
    amount: 600,
    currency: "USD",
    destination: "4627055710465997",
    starting_date: datetime(day: 19, month: 11, year: 2024),
    payment_day_number: 1
  )
)

#agreement_conditions(
  agreement_conditions_data: (
    starting_date: datetime(day: 19, month: 11, year: 2024),
    ending_date: datetime(day: 19, month: 11, year: 2025)
  )
)

#responsibility()

#other_conditions(
  other_conditions_data: (
    min_notice_days_for_visit: 3,
    all_tenants: ("Демчук Назар Ігорович", "Самойленко Марта Юріївна"),
    allowed_animals: (
      "собака породи Чіхуахуа: 1 шт.",
      "кіт породи Персицької: 2 шт."
    ),
  )
)

#signatures()