module Main exposing (Model, Msg, init, subscriptions, update, view)

import Browser
import Html exposing (..)
import Html.Attributes exposing (class, id)
import Icons
import Navbar exposing (NavbarItem, navbar)
import Pages.Ranking
import Pages.Welcome
import Toolbar exposing (toolbar)


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , view = view
        , update = update
        , subscriptions = subscriptions
        }


type alias Model =
    { count : Int
    , options : List (NavbarItem Msg)
    , selectedReport : Report
    , searchErrorMessage : Maybe String
    , searchTextMessage : String
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( Model 0
        [ NavbarItem "Inicio" (SelectedPage Home) Icons.home (Just "home")
        , NavbarItem "Ranking" (SelectedPage Ranking) Icons.award (Just "ranking")
        , NavbarItem "Ranking por Hora" (SelectedPage HourlyRanking) Icons.clock (Just "hourly-ranking")
        ]
        Home
        Nothing
        "1970"
    , Cmd.none
    )


type Report
    = Ranking
    | HourlyRanking
    | Home


type Msg
    = SelectedPage Report
    | NavTextChanged String
    | FormSubmit String


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        SelectedPage page ->
            ( { model | selectedReport = page }, Cmd.none )

        NavTextChanged newText ->
            let
                year =
                    String.toInt newText
                error =
                    case year of
                        Just newYear ->
                            if newYear <= 1979 && newYear >= 1970 then
                                Nothing
                            else Just "Not in bounds"
                        Nothing -> Just "Not a year"
            in
            ( { model | searchTextMessage = newText, searchErrorMessage = error }, Cmd.none )

        FormSubmit newText ->
            ( { model | searchTextMessage = newText }, Cmd.none )


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none


view : Model -> Html Msg
view model =
    div [ class "d-flex w-100 h-100" ]
        [ navbar <| model.options
        , div
            [ id "page-canvas"
            , class "w-100 h-100"
            ]
            [ toolbar
                { inputChange = NavTextChanged
                , formSubmit = FormSubmit
                , inputText = model.searchTextMessage
                , title = "Main Page"
                }
            , div [ class "page-place" ]
                [ div
                    [ class "jumbotron container jumbotron-fluid bg-transparent tab-content"
                    , id "#content-pane"
                    ]
                    [ div
                        [ class "tab-pane fade"
                        , id "home"
                        ]
                        [ Pages.Welcome.page model.options ]
                    , div
                        [ class "tab-pane fade"
                        , id "ranking"
                        ]
                        [ Pages.Ranking.page ]
                    , div
                        [ class "tab-pane fade"
                        , id "hourly-ranking"
                        ]
                        [ Pages.Ranking.page ]
                    ]
                ]
            ]
        ]
