module Main exposing (..)
import Element exposing (..)
import Html exposing (Html)
import Browser
import Maybe exposing (withDefault)
import Navbar exposing (NavbarState)
import Toolbar
import Theme.Icons
import Update exposing (..)
import Navbar exposing (..)
import Theme.Body
import Element.Background
import Toolbar exposing (ToolbarInnerMsg)

type Page = 
    Any

type QueryOption
    = Testing
    | Race

type Msg = 
    NavbarInnerMsg Navbar.NavInternalMsg
    | ToolbarInnerMsg ( Toolbar.ToolbarInnerMsg QueryOption )    | ToolbarSearchIssued String
    | SimulStarted 
    | QueryOptionSelected QueryOption
    | OnPage Page

type alias Model =
    { navModel: Navbar.NavModel Page
    , toolbarModel: Toolbar.ToolbarModel QueryOption }

update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        NavbarInnerMsg inner ->
            let
                ( navModel, cmd ) = Navbar.update model.navModel inner
                newModel = { model | navModel = navModel }
            in
                ( newModel, cmd |> Cmd.map NavbarInnerMsg )

        ToolbarInnerMsg inner ->
            let
                ( toolbarModel, cmd ) = Toolbar.update inner model.toolbarModel
            in
                ( { model | toolbarModel = toolbarModel }, cmd |> Cmd.map ToolbarInnerMsg )

        ToolbarSearchIssued search ->
            let
                _ = Debug.log "Issued Search: " search
            in
                ( model, Cmd.none ) 

        _ -> ( model, Cmd.none )


onPageSelection: Page -> Int -> UpdateMsg Msg
onPageSelection page index = 
    Batch [ OnPage page 
          , NavbarInnerMsg <| Navbar.PageChanged index ]

onSimulPressed: UpdateMsg Msg
onSimulPressed = Batch [ SimulStarted
                       , NavbarInnerMsg <| Navbar.SimulStarted ]

navbarTranslator : { onInternal : Navbar.NavInternalMsg -> UpdateMsg Msg, onPage : Page -> Int -> UpdateMsg Msg, onSimul : UpdateMsg Msg }
navbarTranslator = 
    { onInternal = NavbarInnerMsg >> Single
    , onPage = onPageSelection
    , onSimul = onSimulPressed }


toolbarTranslator : { onInternal : ToolbarInnerMsg QueryOption -> UpdateMsg Msg, onSearchIssued : String -> UpdateMsg Msg, onNavbarToggled : UpdateMsg Msg, onOptionSelected : QueryOption -> UpdateMsg Msg }
toolbarTranslator = 
    { onInternal = ToolbarInnerMsg >> Single
    , onSearchIssued = ToolbarSearchIssued >> Single
    , onNavbarToggled = 
        Batch [ NavbarInnerMsg Navbar.StateToggled 
              , ToolbarInnerMsg Toolbar.NavbarEnvToggled ] 
    , onOptionSelected = 
        \option -> Batch [ 
            ToolbarInnerMsg <| Toolbar.OptionEnvSelected option,
            QueryOptionSelected option
        ]}

view : Model -> Html (UpdateMsg Msg)
view { navModel, toolbarModel } =
    let
        navbarState = navModel.navState
    in
        layout [ Element.Background.color Theme.Body.background ]
            <| row [ height fill 
                   , width fill
                   , Element.Background.color Theme.Body.background ]
                [ Element.map (Navbar.translate navbarTranslator) 
                  <| Navbar.sidebar [] navModel 
                , column [ width fill
                         , height fill
                         , scrollbars ]
                         [ Element.map (Toolbar.translate toolbarTranslator ) 
                           <| Toolbar.toolbar toolbarModel ]
                ]

main : Program () Model (UpdateMsg Msg)
main =
    Browser.element
        { init = ( \_ -> ( Model
                            ( Navbar.init [ { pageInfo = Any, label = "Ganadores Le Mans", icon = Theme.Icons.flag }
                                          , { pageInfo = Any, label = "Loquesea", icon = Theme.Icons.flag }
                                          , { pageInfo = Any, label = "Ganadores Le Mans", icon = Theme.Icons.flag }
                                          , { pageInfo = Any, label = "Ganadores Le Mans", icon = Theme.Icons.flag } ] )
                            ( Toolbar.init [
                                { label = "Ensayo" 
                                , onSelected = Testing },
                                { label = "Carrera" 
                                , onSelected = Race }
                            ] True "Some Title" (Just "Some Subtitle") ), Cmd.none) )
        , view = view
        , update = updateMsg update
        , subscriptions = subscriptions
        }

subscriptions : Model -> Sub (UpdateMsg Msg)
subscriptions model =
    Sub.map (Single << NavbarInnerMsg) <| Navbar.subscriptions model.navModel 