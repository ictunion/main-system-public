@module external styles: {..} = "./MemberDetail/styles.module.scss"


open Data
open Belt

module Actions = {
  open MemberData

  module Send = {
    type acceptTabs =
      | Create
      | Pair

    @react.component
    let make = (~modal, ~api, ~id, ~template) => {
      let (error, setError) = React.useState(() => None)
      let sendEmail = (_: JsxEvent.Mouse.t) => {
        let req =
          api->Api.postJson(
            ~path="/members/" ++ Uuid.toString(id) ++ "/welcome_email",
            ~decoder=Api.Decode.acceptedResponse,
            ~body=MemberData.Encode.newEmailInfo(
                template,
            ),
          )

        req->Future.get(res => {
          switch res {
          | Ok(_) => {
              Modal.Interface.closeModal(modal)
            }
          | Error(e) => setError(_ => Some(e))
          }
        })

      }

      <Modal.Content>
          <div className={styles["modalBody"]}>
            <p> {React.string("Send email to the new member")} </p>
            {switch error {
            | None => React.null
            | Some(err) => <Message.Error> {React.string(err->Api.showError)} </Message.Error>
            }}
          </div>
          <Button.Panel>
            <Button onClick={_ => modal->Modal.Interface.closeModal}>
              {React.string("Cancel")}
            </Button>
            <Button variant=Button.Cta onClick=sendEmail> {React.string("Send email")} </Button>
          </Button.Panel>
      </Modal.Content>
    }
  }

  let sendModal = (~modal, ~api, ~id, ~template): Modal.modalContent => {
    title: "Send email",
    content: <Send modal api id template/>,
  }

  @react.component
  let make = (~status, ~modal, ~api, ~id, ~template) => {
    switch status {
    | NewMember =>
      <Button.Panel>
        <Button
          variant=Button.Cta
          onClick={_ =>
            modal->Modal.Interface.openModal(sendModal(~modal, ~api, ~id, ~template))}>
          {React.string("Send")}
        </Button>
      </Button.Panel>
    | CurrentMember => React.null
    | PastMember => React.null
    }
  }
}



@react.component
let make = (~api, ~id, ~modal) => {
    
    let en: string = 
`<mjml>
    <mj-head>
        <mj-attributes>
        <mj-text line-height="1.5"
                font-size="15px"
                color="#000000"
                font-family="helvetica"
                padding="5px 25px"
                align="justify" />
        <mj-button background-color="#FFC832"
                    color="#000000"
                    border-radius="0"
                    width="100%"
                    height="60px"
                    font-size="24px"
                    padding="10px 25px" />
        <mj-divider border-color="#237fa8"  padding="10px 0"></mj-divider>
        </mj-attributes>
    </mj-head>
    <mj-body>
        <mj-section>
        <mj-column>

            <mj-image width="200px" src="https://ictunion.cz/images/logo.png"></mj-image>

            <mj-divider></mj-divider>

            <mj-text font-size="24px">
                Hello,
            </mj-text>

            <mj-text>
                Welcome to the Trade Union of Workers in ICT. I'm so glad you've joined us!
            </mj-text>
            <mj-text>
                This email is for membership dues only, either you've already heard from someone about how we operate or you'll hear from them soon. I'm sure you will also receive an email with basic information. If you have any questions, feel free to email this email (info@ictunion.cz) or call 737 035 289.
            </mj-text>

            <mj-text>
                Basic information about membership fees<br/>
                The standard membership fee (based on union rules) is 1% of net pay up to a maximum of <b>300 CZK per month</b>. For students, pensioners, unemployed, and members on maternity/parental leave the fee is reduced to 25 CZK/month.
                You can also send a solidarity membership fee. You can send an additional amount up to 100% of the maximum contribution, i.e. up to CZK 600.
                Note: net salary is the salary with all bonuses - you can calculate it as the average of the last 3 salaries.
            </mj-text>

            <mj-text>
                Why dues are important<br/>
                We send 20% of all membership dues to the OS PPP, of which we are a member organisation. The membership dues then pay for a union lawyer, which each of us is entitled to if needed. 80% of the dues stay with our union and can fund our activities.
            </mj-text>

            <mj-text>
                Specific information for you<br/>
                Please send your membership dues to our union's bank account by the 20th of the month: <b>2802034324 / 2010</b>. I recommend that you set up a repeating order for this payment.
            </mj-text>

            <mj-text>
                For easier administration of membership dues, each member receives a variable symbol. Please send your membership fee payment under this variable symbol: <b>{{variable_symbol}}</b> and constant symbol: <b>4710</b>
            </mj-text>

            <mj-text>
                If you have any questions or need more information, feel free to contact me.
            </mj-text>

            <mj-text>
                Invitation to signal channel:</br>
                <b>https://signal.group/#CjQKILmAKt_H0ndPmr31v6IOPqG5c36dNPp0L-3xGnJDpq2nEhAaoddgIDylajThnT21XKHu</b>
            </mj-text>

            <mj-text>
                Trade union wiki:<br/>
                We have also launched a trade union wiki page wiki.ictunion.cz. You can log in to the wiki with your Keycloak account. Your username is the email under which you submitted your application to join the union. You can set a password using the "Forgotten password" function.
            </mj-text>

            <mj-text>
                I look forward to seeing you at one of our meetings or events,
            </mj-text>

            <mj-text>
                On behalf of the ICT Union,
                Daniil Svirin
                Tel: +420 777 350 557
            </mj-text>
        </mj-column>
        </mj-section>
        <mj-hero background-color="#237fa8"  padding="25px 25px">
        <mj-text align="center" color="#FFFFFF" font-size="22px">
            Trade union of workers in ICT
        </mj-text>
        <mj-text align="center" color="#FFFFFF">
            <a href="https://ictunion.cz" style="color: #FFFFFF">ictunion.cz</a> |
            <a href="mailto:support@ictunion.cz" style="color: #FFFFFF">support@ictunion.cz</a> |
            <a href="tel:+420 775 319 271" style="color: #FFFFFF">+420 775 319 271</a>
        </mj-text>
        </mj-hero>
    </mj-body>
</mjml>`

    let cs = 
`<mjml>
    <mj-head>
        <mj-attributes>
            <mj-text line-height="1.5"
                    font-size="15px"
                    color="#000000"
                    font-family="helvetica"
                    padding="5px 25px"
                    align="justify" />
            <mj-button background-color="#FFC832"
                    color="#000000"
                    border-radius="0"
                    width="100%"
                    height="60px"
                    font-size="24px"
                    padding="10px 25px" />
            <mj-divider border-color="#237fa8"  padding="10px 0"></mj-divider>
        </mj-attributes>
    </mj-head>
    <mj-body>
        <mj-section>
            <mj-column>

                <mj-image width="200px" src="https://ictunion.cz/images/logo.png"></mj-image>

                <mj-divider></mj-divider>

                <mj-text font-size="24px">
                    Ahoj,
                </mj-text>

                <mj-text>
                    vítej do Odborové organizace pracujících v ICT. Jsem moc rád, že jsi součástí odborů!
                </mj-text>
                <mj-text>
                    Tento email se týká pouze členských příspěvků, buď se ti už někdo ozval ohledně toho jak fungujeme a nebo se brzy ozve. Určitě ti také dojde email se základními informacemi. V případě jakýchkoliv dotazů, klidně piš na tento email (info@ictunion.cz) nebo na 737 035 289.
                </mj-text>

                <mj-text>
                    Základní informace o členských příspěvcích<br/>
                    Standardní výše členského poplatku (vyplývající z pravidel odborového svazu) je 1% z čisté mzdy do max. výše <b>300 Kč za měsíc</b>. Pro studenty, důchodce, nezaměstnané a členy na mateřské/rodičovské dovolené je poplatek snížen na 25 Kč/měsíc.
                    Můžeš také posílat solidární členský příspěvek. K maximální výši příspěvku, tedy 300 Kč, můžeš přidat až 100% tohoto příspěvku, tzn. posílat až 600 Kč.
                    Poznámka: čistá mzda je mzda se všemi bonusy - vypočítat ji můžeš jako průměr z posledních 3 mezd.
                </mj-text>

                <mj-text>
                    Proč jsou příspěvky důležité<br/>
                    20% všech členských poplatků posíláme OS PPP, jejichž jsme členskou organizací. Z členských poplatků pak platí odborového právníka, na kterého má v případě potřeby každý/á z nás nárok. 80% poplatků zůstává v naší odborové organizaci a můžeme jimi financovat naší činnost.
                </mj-text>

                <mj-text>
                    Specifické informace pro tebe<br/>
                    Prosím o zasílání členských poplatků na bankovní účet naší odborové organizace a to vždy do 20. dne měsíce: <b>2802034324 / 2010</b>. Doporučuju nastavit trvalý příkaz na tuto platbu.
                </mj-text>

                <mj-text>
                    Pro jednodušší administraci členských poplatků dostává každý člen a každá členka svůj variabilní symbol. Platbu členského poplatku posílej pod tímto variabilním symbolem: <b>{{variable_symbol}}</b> a konstantním symbolem: <b>4710</b>
                </mj-text>

                <mj-text>
                    Pokud máš otázky nebo potřebuješ více informací, neváhej se na mě obrátit.
                </mj-text>

                <mj-text>
                    Přihláška do signal kanálu:</br>
                    <b>https://signal.group/#CjQKILmAKt_H0ndPmr31v6IOPqG5c36dNPp0L-3xGnJDpq2nEhAaoddgIDylajThnT21XKHu</b>
                </mj-text>

                <mj-text>
                    Odborová wiki:<br/>
                    Taky jsme spustili odborovou wiki stránku wiki.ictunion.cz. Přihlásit do wiki se můžeš pomocí Keycloak účtu. Tvoje uživatelské jméno je email uvedený na přihlášce do odborů. Heslo si nastavíš pomocí funkce “Zapomenuté heslo”.
                </mj-text>

                <mj-text>
                    Těším se na setkání na nějaké ze schůzek nebo akcí,
                </mj-text>

                <mj-text>
                    Za ICT odbory,
                    Daniil Svirin
                    Tel: +420 777 350 557
                </mj-text>
            </mj-column>
        </mj-section>
        <mj-hero background-color="#237fa8"  padding="25px 25px">
            <mj-text align="center" color="#FFFFFF" font-size="22px">
                Odborová organizace pracujících v ICT
            </mj-text>
            <mj-text align="center" color="#FFFFFF">
                <a href="https://ictunion.cz" style="color: #FFFFFF">ictunion.cz</a> |
                <a href="mailto:support@ictunion.cz" style="color: #FFFFFF">support@ictunion.cz</a> |
                <a href="tel:+420 775 319 271" style="color: #FFFFFF">+420 775 319 271</a>
            </mj-text>
        </mj-hero>
    </mj-body>
</mjml>`

    //new EP needed here, to send email to new member ???

    // back url
    let backPath = "/members/" ++ Uuid.toString(id)

    let (detail: Api.webData<MemberData.detail>, setDetail) = React.useState(RemoteData.init)
    let (text, setText) = React.useState(_ => "");
    let onChange = evt => {
        let emailTxt = ReactEvent.Form.target(evt)["value"]
        setText(_prev => emailTxt);
    }

    React.useEffect0(() => {
        let req = api->Api.getJson(~path="/members/" ++ Uuid.toString(id), ~decoder=MemberData.Decode.detail)
        setDetail(RemoteData.setLoading)

        req->Future.get(res => {
            let detail = RemoteData.fromResult(res)
            setDetail(_ => detail)
            let memberLanguage: string = RemoteData.unwrap(
                detail,
                ~default="cannot happen", 
                member => member.language->Option.getWithDefault("en")
            )
            let template = if (memberLanguage == "cs") { cs } else { en }
            setText(_prev => template)   
        })

        Some(() => Future.cancel(req))
    })
    

    let status = RemoteData.map(detail, MemberData.getStatus)

    <Page requireAnyRole=[ListMembers, ViewMember] mainResource=detail>
        <header className={styles["header"]}>
            <h1 className={styles["title"]}>
                {React.string("Member ")}
                <span className={styles["titleId"]}>
                {switch detail {
                | Success(d) => d.id->Uuid.toString->React.string
                | _ => React.string("...")
                }}
                </span>
            </h1>
            
            <Page.BackButton name="members" path=backPath />
        </header>

        // editable MJML template here
        <textarea onChange name="template" value=text rows=40 />

        {switch status {
        | Success(s) => <Actions status=s modal api id template=text/>
        | _ => React.null
        }}
    </Page>
  }